import { AUTOMATION_SCHEMA_VERSION, type AutomationResult, type ConnectOrder, type PeerDiagnostics } from "./constants.js";

export type StepResult = "PASS" | "FAILED" | "SKIPPED";

export interface ScenarioReport {
  connectOrder: ConnectOrder;
  result: AutomationResult;
  readyReached: boolean;
  completedReached: boolean;
  finalizedDownloads: StepResult;
  peerAEndpointValidation: StepResult;
  peerBEndpointValidation: StepResult;
  pairing: StepResult;
  directoryVerification: StepResult;
  uiClickDispatchDeltaMs: number | null;
  actualEndpointStartDeltaMs: number | null;
  outputDirectory: string;
  error: string | null;
}

export interface AutomationReport {
  schemaVersion: typeof AUTOMATION_SCHEMA_VERSION;
  classification: "automated_single_host_dual_browser";
  evidenceAuthority: "readiness_only";
  physicalTwoDeviceObservationSatisfied: false;
  allRequiredScenariosPassed: boolean;
  sameHost: true;
  separateBrowserProcesses: true;
  fakeMicrophones: true;
  realMicrophonesUsed: false;
  realLanPathProven: false;
  independentHardwareClocksProven: false;
  acousticLatencyMeasured: false;
  authorizedCommit: string;
  result: AutomationResult;
  limitationReason?: string | null;
  scenarios: ScenarioReport[];
  observationSeconds: number;
  notes: string[];
}

export function createScenarioReport(
  order: ConnectOrder,
  outputDirectory: string,
): ScenarioReport {
  return {
    connectOrder: order,
    result: "FAILED",
    readyReached: false,
    completedReached: false,
    finalizedDownloads: "SKIPPED",
    peerAEndpointValidation: "SKIPPED",
    peerBEndpointValidation: "SKIPPED",
    pairing: "SKIPPED",
    directoryVerification: "SKIPPED",
    uiClickDispatchDeltaMs: null,
    actualEndpointStartDeltaMs: null,
    outputDirectory,
    error: null,
  };
}

export function allRequiredScenariosPassed(scenarios: ScenarioReport[]): boolean {
  if (scenarios.length !== 2) {
    return false;
  }
  const orders = scenarios.map((scenario) => scenario.connectOrder);
  if (new Set(orders).size !== 2) {
    return false;
  }
  if (!orders.includes("peer_a_first") || !orders.includes("peer_b_first")) {
    return false;
  }
  return scenarios.every((scenario) => scenario.result === "PASS" && scenarioMayPass(scenario));
}

export function classifyRtpHarnessLimitation(
  peerA: { rtpPreflight?: PeerDiagnostics["rtpPreflight"]; connection: string | null; dataChannel: string | null },
  peerB: { rtpPreflight?: PeerDiagnostics["rtpPreflight"]; connection: string | null; dataChannel: string | null },
): string | null {
  const peers = [peerA, peerB];
  const allExhausted = peers.every((peer) => peer.rtpPreflight?.state === "exhausted");
  const allConnected = peers.every(
    (peer) => peer.connection === "connected" && peer.dataChannel === "open",
  );
  const boundedWindowCompleted = peers.every(
    (peer) =>
      (peer.rtpPreflight?.attempts ?? 0) >= 1 &&
      (peer.rtpPreflight?.elapsed_ms ?? 0) >= 25_000,
  );
  const noRtpSeen = peers.every(
    (peer) => !peer.rtpPreflight?.inbound_audio_seen && !peer.rtpPreflight?.outbound_audio_seen,
  );
  if (allExhausted && allConnected && boundedWindowCompleted && noRtpSeen) {
    return "rtp_audio_stats_unavailable_under_fake_capture";
  }
  return null;
}

export function buildAutomationReport(input: {
  authorizedCommit: string;
  result: AutomationResult;
  limitationReason?: string | null;
  scenarios: ScenarioReport[];
  observationSeconds: number;
  notes?: string[];
}): AutomationReport {
  const requiredPassed = allRequiredScenariosPassed(input.scenarios);
  const derivedResult: AutomationResult = requiredPassed ? "PASS" : input.result;
  return {
    schemaVersion: AUTOMATION_SCHEMA_VERSION,
    classification: "automated_single_host_dual_browser",
    evidenceAuthority: "readiness_only",
    physicalTwoDeviceObservationSatisfied: false,
    allRequiredScenariosPassed: requiredPassed,
    sameHost: true,
    separateBrowserProcesses: true,
    fakeMicrophones: true,
    realMicrophonesUsed: false,
    realLanPathProven: false,
    independentHardwareClocksProven: false,
    acousticLatencyMeasured: false,
    authorizedCommit: input.authorizedCommit,
    result: derivedResult,
    limitationReason: input.limitationReason ?? null,
    scenarios: input.scenarios,
    observationSeconds: input.observationSeconds,
    notes: input.notes ?? [
      "Automation acknowledgement exercises the UI only; it does not claim physical headphones.",
      "PASS means the supported single-host dual-browser flow completed; it is not physical two-device evidence.",
    ],
  };
}

export function classificationNeverClaimsPhysical(report: AutomationReport): boolean {
  return (
    report.evidenceAuthority === "readiness_only" &&
    report.physicalTwoDeviceObservationSatisfied === false &&
    report.realLanPathProven === false &&
    report.independentHardwareClocksProven === false &&
    report.acousticLatencyMeasured === false &&
    report.classification === "automated_single_host_dual_browser"
  );
}

export function overallPassRequiresAllScenarios(scenarios: ScenarioReport[]): boolean {
  return allRequiredScenariosPassed(scenarios);
}

export function scenarioMayPass(report: ScenarioReport): boolean {
  return (
    report.readyReached &&
    report.completedReached &&
    report.finalizedDownloads === "PASS" &&
    report.peerAEndpointValidation === "PASS" &&
    report.peerBEndpointValidation === "PASS" &&
    report.pairing === "PASS" &&
    report.directoryVerification === "PASS" &&
    report.uiClickDispatchDeltaMs !== null &&
    report.actualEndpointStartDeltaMs !== null
  );
}
