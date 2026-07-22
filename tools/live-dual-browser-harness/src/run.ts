import { copyFileSync, existsSync } from "node:fs";
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
import { automationLayout, resolveAutomationRunDir } from "./paths.js";
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
  synchronizedObservationStart,
  type PeerSession,
} from "./peer-runner.js";
import { ProcessManager } from "./process-manager.js";
import { buildAutomationReport, type ScenarioReport } from "./report.js";
import { ensureFakeAudioFixtures } from "./wav.js";
import { pairLiveEndpoints, validateLiveEndpoint, verifyLiveDirectory } from "./validation.js";

async function runScenario(options: {
  order: ConnectOrder;
  correlationId: string;
  fakeAudio: { peerA: string; peerB: string };
  layout: ReturnType<typeof automationLayout>;
}): Promise<{ scenario: ScenarioReport; peerADiagnostics: PeerDiagnostics; peerBDiagnostics: PeerDiagnostics }> {
  const userDataA = createTempUserDataDir("echlub-live-a-");
  const userDataB = createTempUserDataDir("echlub-live-b-");
  if (userDataA === userDataB) {
    throw new Error("peer browser user-data directories must be distinct");
  }

  let peerA: PeerSession | null = null;
  let peerB: PeerSession | null = null;
  try {
    peerA = await launchPeer({
      role: "peer_a",
      correlationId: options.correlationId,
      fakeAudioPath: options.fakeAudio.peerA,
      userDataDir: userDataA,
      diagnosticsDir: options.layout.diagnosticsDir,
    });
    peerB = await launchPeer({
      role: "peer_b",
      correlationId: options.correlationId,
      fakeAudioPath: options.fakeAudio.peerB,
      userDataDir: userDataB,
      diagnosticsDir: options.layout.diagnosticsDir,
    });

    await runConnectOrder(options.order, peerA, peerB);
    await assertPeerReadyStates(peerA, peerB);
    const observationStartDeltaMs = await synchronizedObservationStart(peerA, peerB);
    assertObservationStartWindow(observationStartDeltaMs, OBSERVATION_START_WINDOW_MS);
    await assertPeerCompletedStates(peerA, peerB);
    await saveDownloads(peerA, peerB, options.layout.peerAJson, options.layout.peerBJson);
    assertEndpointNegotiationRoles(options.layout.peerAJson, options.layout.peerBJson);

    const peerADiagnostics = await capturePeerArtifacts(peerA, options.layout.diagnosticsDir, "peer-a");
    const peerBDiagnostics = await capturePeerArtifacts(peerB, options.layout.diagnosticsDir, "peer-b");

    return {
      scenario: {
        connectOrder: options.order,
        result: "PASS",
        readyReached: true,
        completedReached: true,
        observationStartDeltaMs,
        error: null,
      },
      peerADiagnostics,
      peerBDiagnostics,
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    const peerADiagnostics =
      peerA !== null
        ? await capturePeerArtifacts(peerA, options.layout.diagnosticsDir, "peer-a")
        : emptyDiagnostics(message);
    const peerBDiagnostics =
      peerB !== null
        ? await capturePeerArtifacts(peerB, options.layout.diagnosticsDir, "peer-b")
        : emptyDiagnostics(message);
    return {
      scenario: {
        connectOrder: options.order,
        result: "FAILED",
        readyReached: false,
        completedReached: false,
        observationStartDeltaMs: null,
        error: message,
      },
      peerADiagnostics,
      peerBDiagnostics,
    };
  } finally {
    if (peerA) await closePeer(peerA);
    if (peerB) await closePeer(peerB);
    removeTempUserDataDir(userDataA);
    removeTempUserDataDir(userDataB);
  }
}

function emptyDiagnostics(error: string): PeerDiagnostics {
  return {
    phase: null,
    connection: null,
    ice: null,
    dataChannel: null,
    samples: null,
    probes: null,
    error,
    console: [],
    pageErrors: [],
  };
}

function writeDiagnosticsBundle(
  diagnosticsDir: string,
  scenarios: Array<{
    order: ConnectOrder;
    peerA: PeerDiagnostics;
    peerB: PeerDiagnostics;
  }>,
): void {
  writeJson(join(diagnosticsDir, "browser-console.json"), scenarios);
  const peerATrace = join(diagnosticsDir, "peer-a-trace.zip");
  const combinedTrace = join(diagnosticsDir, "playwright-trace.zip");
  if (existsSync(peerATrace)) {
    copyFileSync(peerATrace, combinedTrace);
  }
}

async function main(): Promise<number> {
  const runId = generateAutomationRunId();
  const layout = automationLayout(resolveAutomationRunDir(runId));
  ensureDir(layout.runDir);
  ensureDir(layout.rawDir);
  ensureDir(layout.pairedDir);
  ensureDir(layout.diagnosticsDir);
  const processLog = join(layout.diagnosticsDir, "process-log.txt");
  const processes = new ProcessManager(processLog);

  const fakeAudio = ensureFakeAudioFixtures();
  const scenarios: ScenarioReport[] = [];
  const diagnosticsBundle: Array<{
    order: ConnectOrder;
    peerA: PeerDiagnostics;
    peerB: PeerDiagnostics;
  }> = [];
  let finalResult: "PASS" | "FAILED" = "PASS";
  let failureMessage: string | null = null;
  let authorizedCommit = "unknown";

  try {
    authorizedCommit = processes.verifyGitCommit();
    await processes.startServers(authorizedCommit);

    for (const order of ["peer_a_first", "peer_b_first"] as ConnectOrder[]) {
      const correlationId = generateCorrelationId(`${runId}:${order}`);
      const outcome = await runScenario({ order, correlationId, fakeAudio, layout });
      scenarios.push(outcome.scenario);
      diagnosticsBundle.push({
        order,
        peerA: outcome.peerADiagnostics,
        peerB: outcome.peerBDiagnostics,
      });
      if (outcome.scenario.result !== "PASS") {
        finalResult = "FAILED";
        failureMessage = outcome.scenario.error;
        break;
      }
    }

    if (finalResult === "PASS") {
      if (!existsSync(layout.peerAJson) || !existsSync(layout.peerBJson)) {
        throw new Error("missing finalized endpoint downloads");
      }
      validateLiveEndpoint(layout.peerAJson);
      validateLiveEndpoint(layout.peerBJson);
      pairLiveEndpoints(layout.peerAJson, layout.peerBJson, layout.pairedDir);
      verifyLiveDirectory(layout.pairedDir);
    }
  } catch (error) {
    finalResult = "FAILED";
    failureMessage = error instanceof Error ? error.message : String(error);
  } finally {
    writeDiagnosticsBundle(layout.diagnosticsDir, diagnosticsBundle);
    await processes.terminateAllAndWait();
  }

  const report = buildAutomationReport({
    authorizedCommit,
    result: finalResult,
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
