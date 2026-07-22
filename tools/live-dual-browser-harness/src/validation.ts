import { spawnSync } from "node:child_process";
import { REPO_ROOT } from "./constants.js";

export function runCargoReport(args: string[]): { status: number; stdout: string; stderr: string } {
  const result = spawnSync("cargo", ["run", "-p", "echlub-performance-report", "--", ...args], {
    cwd: REPO_ROOT,
    encoding: "utf8",
  });
  return {
    status: result.status ?? 1,
    stdout: result.stdout ?? "",
    stderr: result.stderr ?? "",
  };
}

export function validateLiveEndpoint(path: string): void {
  const result = runCargoReport(["validate-live-endpoint", path]);
  if (result.status !== 0) {
    throw new Error(`validate-live-endpoint failed for ${path}\n${result.stderr}\n${result.stdout}`);
  }
}

export function pairLiveEndpoints(peerA: string, peerB: string, outputDir: string): void {
  const result = runCargoReport(["pair-live-endpoints", peerA, peerB, "--output", outputDir]);
  if (result.status !== 0) {
    throw new Error(`pair-live-endpoints failed\n${result.stderr}\n${result.stdout}`);
  }
}

export function verifyLiveDirectory(dir: string): void {
  const result = runCargoReport(["verify-live-directory", dir]);
  if (result.status !== 0) {
    throw new Error(`verify-live-directory failed for ${dir}\n${result.stderr}\n${result.stdout}`);
  }
}
