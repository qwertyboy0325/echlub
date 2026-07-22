import { resolve } from "node:path";
import { type ConnectOrder, REPO_ROOT } from "./constants.js";

const OWNER_IMPORT_ROOT = resolve(REPO_ROOT, ".local/live-observation-import");
const AUTOMATION_ROOT = resolve(REPO_ROOT, ".local/live-automation");

export function resolveAutomationRunDir(runId: string): string {
  return resolve(AUTOMATION_ROOT, runId);
}

export function connectOrderDirName(order: ConnectOrder): string {
  return order === "peer_a_first" ? "peer-a-first" : "peer-b-first";
}

export function assertSafeAutomationOutputPath(targetPath: string): void {
  const resolved = resolve(targetPath);
  const ownerRoot = resolve(OWNER_IMPORT_ROOT);
  if (resolved === ownerRoot || resolved.startsWith(`${ownerRoot}/`)) {
    throw new Error(`automation output path forbidden under owner import directory: ${resolved}`);
  }
  const automationRoot = resolve(AUTOMATION_ROOT);
  if (resolved !== automationRoot && !resolved.startsWith(`${automationRoot}/`)) {
    throw new Error(`automation output path must resolve under .local/live-automation/: ${resolved}`);
  }
}

export function runLayout(runDir: string) {
  assertSafeAutomationOutputPath(runDir);
  return {
    runDir,
    scenariosDir: resolve(runDir, "scenarios"),
    diagnosticsDir: resolve(runDir, "diagnostics"),
    processLog: resolve(runDir, "diagnostics/process-log.txt"),
    automationReport: resolve(runDir, "automation-report.json"),
  };
}

export function scenarioLayout(runDir: string, order: ConnectOrder) {
  const scenarioDir = resolve(runDir, "scenarios", connectOrderDirName(order));
  assertSafeAutomationOutputPath(scenarioDir);
  return {
    scenarioDir,
    order,
    rawDir: resolve(scenarioDir, "raw"),
    pairedDir: resolve(scenarioDir, "paired"),
    diagnosticsDir: resolve(scenarioDir, "diagnostics"),
    peerAJson: resolve(scenarioDir, "raw/peer-a.json"),
    peerBJson: resolve(scenarioDir, "raw/peer-b.json"),
  };
}

/** @deprecated use runLayout / scenarioLayout */
export function automationLayout(runDir: string) {
  return runLayout(runDir);
}

export { AUTOMATION_ROOT, OWNER_IMPORT_ROOT };
