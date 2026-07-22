import { AUTOMATION_SCHEMA_VERSION, type AutomationResult, type ConnectOrder } from "./constants.js";

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

export function buildAutomationReport(input: {
  authorizedCommit: string;
  result: AutomationResult;
  scenarios: ScenarioReport[];
  observationSeconds: number;
  notes?: string[];
}): AutomationReport {
  const allRequiredScenariosPassed =
    input.scenarios.length > 0 && input.scenarios.every((scenario) => scenario.result === "PASS");
  return {
    schemaVersion: AUTOMATION_SCHEMA_VERSION,
    classification: "automated_single_host_dual_browser",
    evidenceAuthority: "readiness_only",
    physicalTwoDeviceObservationSatisfied: false,
    allRequiredScenariosPassed,
    sameHost: true,
    separateBrowserProcesses: true,
    fakeMicrophones: true,
    realMicrophonesUsed: false,
    realLanPathProven: false,
    independentHardwareClocksProven: false,
    acousticLatencyMeasured: false,
    authorizedCommit: input.authorizedCommit,
    result: input.result,
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
  return scenarios.length > 0 && scenarios.every((scenario) => scenario.result === "PASS");
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
