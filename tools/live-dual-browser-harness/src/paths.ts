import { resolve } from "node:path";
import { REPO_ROOT } from "./constants.js";

const OWNER_IMPORT_ROOT = resolve(REPO_ROOT, ".local/live-observation-import");
const AUTOMATION_ROOT = resolve(REPO_ROOT, ".local/live-automation");

export function resolveAutomationRunDir(runId: string): string {
  return resolve(AUTOMATION_ROOT, runId);
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

export function automationLayout(runDir: string) {
  assertSafeAutomationOutputPath(runDir);
  return {
    runDir,
    rawDir: resolve(runDir, "raw"),
    pairedDir: resolve(runDir, "paired"),
    diagnosticsDir: resolve(runDir, "diagnostics"),
    automationReport: resolve(runDir, "automation-report.json"),
    peerAJson: resolve(runDir, "raw/peer-a.json"),
    peerBJson: resolve(runDir, "raw/peer-b.json"),
  };
}

export { AUTOMATION_ROOT, OWNER_IMPORT_ROOT };
