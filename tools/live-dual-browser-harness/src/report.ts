import { AUTOMATION_SCHEMA_VERSION, type AutomationResult, type ConnectOrder } from "./constants.js";

export interface ScenarioReport {
  connectOrder: ConnectOrder;
  result: AutomationResult;
  readyReached: boolean;
  completedReached: boolean;
  observationStartDeltaMs: number | null;
  error: string | null;
}

export interface AutomationReport {
  schemaVersion: typeof AUTOMATION_SCHEMA_VERSION;
  classification: "automated_single_host_dual_browser";
  evidenceAuthority: "readiness_only";
  physicalTwoDeviceObservationSatisfied: false;
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

export function buildAutomationReport(input: {
  authorizedCommit: string;
  result: AutomationResult;
  scenarios: ScenarioReport[];
  observationSeconds: number;
  notes?: string[];
}): AutomationReport {
  return {
    schemaVersion: AUTOMATION_SCHEMA_VERSION,
    classification: "automated_single_host_dual_browser",
    evidenceAuthority: "readiness_only",
    physicalTwoDeviceObservationSatisfied: false,
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
