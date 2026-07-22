import { join } from "node:path";
import {
  ensureDir,
  generateAutomationRunId,
  generateCorrelationId,
  OBSERVATION_SECONDS,
  OBSERVATION_START_WINDOW_MS,
  writeJson,
  type ConnectOrder,
  type PeerDiagnostics,
} from "./constants.js";
import { calculateActualEndpointStartDeltaMs } from "./endpoint-timing.js";
import { resolveAutomationRunDir, runLayout, scenarioLayout } from "./paths.js";
import {
  assertEndpointNegotiationRoles,
  assertObservationStartWindow,
  assertPeerCompletedStates,
  assertPeerReadyStates,
  capturePeerArtifacts,
  closePeer,
  createTempUserDataDir,
  launchPeer,
  removeTempUserDataDir,
  runConnectOrder,
  saveDownloads,
  snapshotPeerDiagnostics,
  synchronizedObservationStart,
  type PeerSession,
} from "./peer-runner.js";
import { ProcessManager } from "./process-manager.js";
import {
  allRequiredScenariosPassed,
  buildAutomationReport,
  createScenarioReport,
  mayClassifyScenarioAsHarnessLimitation,
  scenarioMayPass,
  type ScenarioReport,
} from "./report.js";
import { ensureFakeAudioFixtures } from "./wav.js";
import {
  assertFinalizedDownloadsExist,
  pairLiveEndpoints,
  validateLiveEndpoint,
  verifyLiveDirectory,
} from "./validation.js";

function emptyDiagnostics(error: string): PeerDiagnostics {
  return {
    phase: null,
    connection: null,
    ice: null,
    dataChannel: null,
    samples: null,
    probes: null,
    remoteTrackLive: null,
    error,
    console: [],
    pageErrors: [],
  };
}

function markStepFailed(report: ScenarioReport, step: keyof Pick<
  ScenarioReport,
  | "finalizedDownloads"
  | "peerAEndpointValidation"
  | "peerBEndpointValidation"
  | "pairing"
  | "directoryVerification"
>): void {
  if (report[step] === "SKIPPED") {
    report[step] = "FAILED";
  }
}

async function writeScenarioDiagnostics(
  layout: ReturnType<typeof scenarioLayout>,
  runProcessLog: string,
  peerA: PeerDiagnostics,
  peerB: PeerDiagnostics,
): Promise<void> {
  writeJson(join(layout.diagnosticsDir, "browser-console.json"), {
    connectOrder: layout.order,
    peerA,
    peerB,
    rtp_preflight: {
      peerA: peerA.rtpPreflight ?? null,
      peerB: peerB.rtpPreflight ?? null,
    },
  });
  writeJson(join(layout.diagnosticsDir, "process-log-reference.json"), {
    processLog: runProcessLog,
  });
}

async function runScenario(options: {
  order: ConnectOrder;
  correlationId: string;
  fakeAudio: { peerA: string; peerB: string };
  runDir: string;
  runProcessLog: string;
}): Promise<ScenarioReport> {
  const layout = scenarioLayout(options.runDir, options.order);
  ensureDir(layout.rawDir);
  ensureDir(layout.pairedDir);
  ensureDir(layout.diagnosticsDir);

  const report = createScenarioReport(options.order, layout.scenarioDir);
  const userDataA = createTempUserDataDir("echlub-live-a-");
  const userDataB = createTempUserDataDir("echlub-live-b-");

  let peerA: PeerSession | null = null;
  let peerB: PeerSession | null = null;
  let peerADiagnostics = emptyDiagnostics("peer A not launched");
  let peerBDiagnostics = emptyDiagnostics("peer B not launched");
  let diagnosticCaptureFailed = false;
  let cleanupFailed = false;
  let originalScenarioError: string | null = null;

  try {
    if (userDataA === userDataB) {
      throw new Error("peer browser user-data directories must be distinct");
    }
    peerA = await launchPeer({
      role: "peer_a",
      correlationId: options.correlationId,
      fakeAudioPath: options.fakeAudio.peerA,
      userDataDir: userDataA,
      diagnosticsDir: layout.diagnosticsDir,
    });
    peerB = await launchPeer({
      role: "peer_b",
      correlationId: options.correlationId,
      fakeAudioPath: options.fakeAudio.peerB,
      userDataDir: userDataB,
      diagnosticsDir: layout.diagnosticsDir,
    });

    await runConnectOrder(options.order, peerA, peerB);
    await assertPeerReadyStates(peerA, peerB);
    report.readyReached = true;

    const uiClickDispatchDeltaMs = await synchronizedObservationStart(peerA, peerB);
    assertObservationStartWindow(uiClickDispatchDeltaMs, OBSERVATION_START_WINDOW_MS);
    report.uiClickDispatchDeltaMs = uiClickDispatchDeltaMs;

    await assertPeerCompletedStates(peerA, peerB);
    report.completedReached = true;

    await saveDownloads(peerA, peerB, layout.peerAJson, layout.peerBJson);
    assertFinalizedDownloadsExist(layout.peerAJson, layout.peerBJson);
    report.finalizedDownloads = "PASS";

    assertEndpointNegotiationRoles(layout.peerAJson, layout.peerBJson);

    report.actualEndpointStartDeltaMs = calculateActualEndpointStartDeltaMs(
      layout.peerAJson,
      layout.peerBJson,
    );

    validateLiveEndpoint(layout.peerAJson);
    report.peerAEndpointValidation = "PASS";
    validateLiveEndpoint(layout.peerBJson);
    report.peerBEndpointValidation = "PASS";

    pairLiveEndpoints(layout.peerAJson, layout.peerBJson, layout.pairedDir);
    report.pairing = "PASS";

    verifyLiveDirectory(layout.pairedDir);
    report.directoryVerification = "PASS";

    if (!scenarioMayPass(report)) {
      throw new Error("scenario gate incomplete despite successful validation steps");
    }
    report.result = "PASS";
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    originalScenarioError = message;
    report.error = message;
    report.result = "FAILED";

    if (report.completedReached && report.finalizedDownloads === "SKIPPED") {
      report.finalizedDownloads = "FAILED";
    }
    if (report.finalizedDownloads === "PASS" && report.peerAEndpointValidation === "SKIPPED") {
      markStepFailed(report, "peerAEndpointValidation");
    }
    if (report.peerAEndpointValidation === "PASS" && report.peerBEndpointValidation === "SKIPPED") {
      markStepFailed(report, "peerBEndpointValidation");
    }
    if (report.peerBEndpointValidation === "PASS" && report.pairing === "SKIPPED") {
      markStepFailed(report, "pairing");
    }
    if (report.pairing === "PASS" && report.directoryVerification === "SKIPPED") {
      markStepFailed(report, "directoryVerification");
    }
  } finally {
    if (peerA !== null && peerB !== null) {
      try {
        peerADiagnostics = await snapshotPeerDiagnostics(peerA);
        peerBDiagnostics = await snapshotPeerDiagnostics(peerB);
      } catch (error) {
        diagnosticCaptureFailed = true;
        const message = error instanceof Error ? error.message : String(error);
        peerADiagnostics = emptyDiagnostics(`peer diagnostic snapshot failed: ${message}`);
        peerBDiagnostics = emptyDiagnostics(`peer diagnostic snapshot failed: ${message}`);
      }
    }

    if (peerA !== null) {
      try {
        peerADiagnostics = await capturePeerArtifacts(
          peerA,
          layout.diagnosticsDir,
          "peer-a",
          peerADiagnostics,
        );
      } catch (error) {
        diagnosticCaptureFailed = true;
        const message = error instanceof Error ? error.message : String(error);
        peerADiagnostics = emptyDiagnostics(`peer A diagnostic capture failed: ${message}`);
      } finally {
        try {
          await closePeer(peerA);
        } catch (error) {
          cleanupFailed = true;
          const message = error instanceof Error ? error.message : String(error);
          peerADiagnostics.error = `${peerADiagnostics.error ?? ""}; close failed: ${message}`.replace(/^; /, "");
        }
        try {
          removeTempUserDataDir(userDataA);
        } catch (error) {
          cleanupFailed = true;
        }
      }
    } else {
      removeTempUserDataDir(userDataA);
    }

    if (peerB !== null) {
      try {
        peerBDiagnostics = await capturePeerArtifacts(
          peerB,
          layout.diagnosticsDir,
          "peer-b",
          peerBDiagnostics,
        );
      } catch (error) {
        diagnosticCaptureFailed = true;
        const message = error instanceof Error ? error.message : String(error);
        peerBDiagnostics = emptyDiagnostics(`peer B diagnostic capture failed: ${message}`);
      } finally {
        try {
          await closePeer(peerB);
        } catch (error) {
          cleanupFailed = true;
          const message = error instanceof Error ? error.message : String(error);
          peerBDiagnostics.error = `${peerBDiagnostics.error ?? ""}; close failed: ${message}`.replace(/^; /, "");
        }
        try {
          removeTempUserDataDir(userDataB);
        } catch (error) {
          cleanupFailed = true;
        }
      }
    } else {
      removeTempUserDataDir(userDataB);
    }

    await writeScenarioDiagnostics(layout, options.runProcessLog, peerADiagnostics, peerBDiagnostics);
    if (diagnosticCaptureFailed || cleanupFailed) {
      report.result = "FAILED";
      report.error =
        originalScenarioError ??
        report.error ??
        (cleanupFailed ? "browser cleanup failed" : "diagnostic capture failed");
    } else if (!report.readyReached && report.result === "FAILED") {
      const limitationReason = mayClassifyScenarioAsHarnessLimitation({
        report,
        peerA: peerADiagnostics,
        peerB: peerBDiagnostics,
        diagnosticCaptureFailed,
        cleanupFailed,
        originalScenarioError,
      });
      if (limitationReason) {
        report.result = "HARNESS_LIMITATION";
        report.error = limitationReason;
      }
    }
  }

  return report;
}

async function main(): Promise<number> {
  const runId = generateAutomationRunId();
  const layout = runLayout(resolveAutomationRunDir(runId));
  ensureDir(layout.runDir);
  ensureDir(layout.scenariosDir);
  ensureDir(layout.diagnosticsDir);

  const processes = new ProcessManager(layout.processLog);
  const fakeAudio = ensureFakeAudioFixtures();
  const scenarios: ScenarioReport[] = [];
  let finalResult: import("./constants.js").AutomationResult = "PASS";
  let limitationReason: string | null = null;
  let failureMessage: string | null = null;
  let cleanupFailed = false;
  let authorizedCommit = "unknown";

  try {
    authorizedCommit = processes.verifyGitCommit();
    await processes.startServers(authorizedCommit);

    for (const order of ["peer_a_first", "peer_b_first"] as ConnectOrder[]) {
      const correlationId = generateCorrelationId(`${runId}:${order}`);
      const scenario = await runScenario({
        order,
        correlationId,
        fakeAudio,
        runDir: layout.runDir,
        runProcessLog: layout.processLog,
      });
      scenarios.push(scenario);
      if (scenario.result !== "PASS" && failureMessage === null) {
        failureMessage = scenario.error;
      }
      if (scenario.result === "HARNESS_LIMITATION" && limitationReason === null) {
        limitationReason = scenario.error;
      }
    }

    if (allRequiredScenariosPassed(scenarios)) {
      finalResult = "PASS";
    } else if (scenarios.length > 0 && scenarios.every((scenario) => scenario.result === "HARNESS_LIMITATION")) {
      finalResult = "HARNESS_LIMITATION";
    } else {
      finalResult = "FAILED";
    }
  } catch (error) {
    finalResult = "FAILED";
    failureMessage = error instanceof Error ? error.message : String(error);
  } finally {
    try {
      await processes.terminateAllAndWait();
    } catch (error) {
      cleanupFailed = true;
      finalResult = "FAILED";
      const cleanupMessage = error instanceof Error ? error.message : String(error);
      failureMessage = failureMessage ?? cleanupMessage;
    }
  }

  if (cleanupFailed) {
    finalResult = "FAILED";
  }

  const report = buildAutomationReport({
    authorizedCommit,
    result: finalResult,
    limitationReason,
    scenarios,
    observationSeconds: OBSERVATION_SECONDS,
    notes: failureMessage ? [`failure: ${failureMessage}`] : undefined,
  });
  writeJson(layout.automationReport, report);
  console.log(`Automation report: ${layout.automationReport}`);
  console.log(`Automation result: ${report.result}`);
  return finalResult === "PASS" ? 0 : 1;
}

main()
  .then((code) => process.exit(code))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });

export { runScenario, scenarioMayPass, allRequiredScenariosPassed };
