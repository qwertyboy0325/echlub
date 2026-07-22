import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
export const HARNESS_ROOT = join(__dirname, "..");
export const REPO_ROOT = join(HARNESS_ROOT, "../..");
export const FIXTURES_DIR = join(HARNESS_ROOT, "fixtures");
export const FAKE_AUDIO_PEER_A = join(FIXTURES_DIR, "fake-peer-a.wav");
export const FAKE_AUDIO_PEER_B = join(FIXTURES_DIR, "fake-peer-b.wav");

export const AUTOMATION_SCHEMA_VERSION = "echlub.live-automation/v1";
export const WEB_ORIGIN = "http://127.0.0.1:5173";
export const LIVE_URL = `${WEB_ORIGIN}/?section=live`;
export const SIGNALING_URL = "ws://127.0.0.1:8080/v1/signaling/ws";
export const OBSERVATION_SECONDS = 60;
export const READY_TIMEOUT_MS = 180_000;
export const COMPLETED_TIMEOUT_MS = 120_000;
export const OBSERVATION_START_WINDOW_MS = 2_000;

export type ConnectOrder = "peer_a_first" | "peer_b_first";
export type AutomationResult = "PASS" | "PARTIAL" | "FAILED" | "HARNESS_LIMITATION";

export interface PeerDiagnostics {
  phase: string | null;
  connection: string | null;
  ice: string | null;
  dataChannel: string | null;
  samples: string | null;
  probes: string | null;
  error: string | null;
  console: string[];
  pageErrors: string[];
  rtpPreflight?: {
    state: string;
    attempts: number;
    inbound_audio_seen: boolean;
    outbound_audio_seen: boolean;
    elapsed_ms: number;
    failure_reason: string | null;
    sanitized_report_shapes: unknown[];
  } | null;
}

export function generateAutomationRunId(now = new Date()): string {
  return now.toISOString().replace(/[:.]/g, "-").replace("T", "T").replace("Z", "Z");
}

export function generateCorrelationId(seed = "echlub-live-automation"): string {
  let hash = 0;
  for (let i = 0; i < seed.length; i += 1) {
    hash = (hash * 31 + seed.charCodeAt(i)) >>> 0;
  }
  const parts = [hash];
  for (let i = 1; i < 4; i += 1) {
    hash = (hash * 1664525 + 1013904223 + i) >>> 0;
    parts.push(hash);
  }
  return parts.map((value) => value.toString(16).padStart(8, "0")).join("");
}

export function ensureDir(path: string): void {
  mkdirSync(path, { recursive: true });
}

export function writeJson(path: string, value: unknown): void {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}
