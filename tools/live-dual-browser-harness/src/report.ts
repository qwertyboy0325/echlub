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

function iceConnected(ice: string | null | undefined): boolean {
  return ice === "connected" || ice === "completed";
}

export function classifyPeerRtpHarnessLimitation(peer: PeerDiagnostics): string | null {
  const pf = peer.rtpPreflight;
  if (!pf || pf.state !== "exhausted") return null;
  if (peer.connection !== "connected") return null;
  if (!iceConnected(peer.ice)) return null;
  if (peer.dataChannel !== "open") return null;
  if (peer.remoteTrackLive !== true) return null;
  if ((pf.attempts ?? 0) < 1 || (pf.elapsed_ms ?? 0) < 25_000) return null;

  const inbound = pf.inbound_audio_seen;
  const outbound = pf.outbound_audio_seen;
  if (inbound && outbound) return null;

  if (!inbound && !outbound) {
    return "bilateral_rtp_audio_stats_unavailable_under_fake_capture";
  }
  if (!inbound) {
    return "inbound_rtp_audio_stats_unavailable_under_fake_capture";
  }
  return "outbound_rtp_audio_stats_unavailable_under_fake_capture";
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

export function classifyRtpHarnessLimitation(peerA: PeerDiagnostics, peerB: PeerDiagnostics): string | null {
  const peerAReason = classifyPeerRtpHarnessLimitation(peerA);
  const peerBReason = classifyPeerRtpHarnessLimitation(peerB);
  if (!peerAReason || !peerBReason) return null;
  if (peerAReason === peerBReason) return peerAReason;
  return "bilateral_rtp_audio_stats_unavailable_under_fake_capture";
}

export const BOUNDED_RTP_PREFLIGHT_SCENARIO_FAILURE = "rtp_audio_stats_unavailable_under_fake_capture";

function peerDiagnosticSnapshotAvailable(peer: PeerDiagnostics): boolean {
  if (peer.error?.includes("snapshot failed") || peer.error?.includes("capture failed")) {
    return false;
  }
  return peer.rtpPreflight?.state === "exhausted" && peer.connection === "connected";
}

function peerHasUnrelatedFailure(peer: PeerDiagnostics): boolean {
  if (peer.pageErrors.length > 0) return true;
  return peer.error !== null && peer.error.length > 0;
}

export interface ScenarioHarnessLimitationInput {
  report: ScenarioReport;
  peerA: PeerDiagnostics;
  peerB: PeerDiagnostics;
  diagnosticCaptureFailed: boolean;
  cleanupFailed: boolean;
  originalScenarioError: string | null;
}

export function mayClassifyScenarioAsHarnessLimitation(
  input: ScenarioHarnessLimitationInput,
): string | null {
  if (input.report.readyReached) return null;
  if (input.report.result !== "FAILED") return null;
  if (input.diagnosticCaptureFailed || input.cleanupFailed) return null;
  if (input.originalScenarioError !== BOUNDED_RTP_PREFLIGHT_SCENARIO_FAILURE) return null;
  if (input.report.finalizedDownloads !== "SKIPPED") return null;
  if (!peerDiagnosticSnapshotAvailable(input.peerA)) return null;
  if (!peerDiagnosticSnapshotAvailable(input.peerB)) return null;
  if (peerHasUnrelatedFailure(input.peerA)) return null;
  if (peerHasUnrelatedFailure(input.peerB)) return null;
  return classifyRtpHarnessLimitation(input.peerA, input.peerB);
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
