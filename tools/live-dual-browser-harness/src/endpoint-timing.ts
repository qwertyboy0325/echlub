import { readFileSync } from "node:fs";
import { OBSERVATION_START_WINDOW_MS } from "./constants.js";

interface EndpointTiming {
  startedAtUtc?: string;
}

function parseStartedAtUtc(path: string): number {
  const raw = readFileSync(path, "utf8");
  const parsed = JSON.parse(raw) as EndpointTiming;
  const startedAt = parsed.startedAtUtc;
  if (typeof startedAt !== "string" || startedAt.trim().length === 0) {
    throw new Error(`missing startedAtUtc in ${path}`);
  }
  const ms = Date.parse(startedAt);
  if (!Number.isFinite(ms)) {
    throw new Error(`invalid startedAtUtc in ${path}: ${startedAt}`);
  }
  return ms;
}

export function calculateActualEndpointStartDeltaMs(
  peerAPath: string,
  peerBPath: string,
  maxMs = OBSERVATION_START_WINDOW_MS,
): number {
  const peerAStart = parseStartedAtUtc(peerAPath);
  const peerBStart = parseStartedAtUtc(peerBPath);
  const deltaMs = Math.abs(peerAStart - peerBStart);
  if (deltaMs > maxMs) {
    throw new Error(`actual endpoint start delta ${deltaMs}ms exceeds ${maxMs}ms`);
  }
  return deltaMs;
}
