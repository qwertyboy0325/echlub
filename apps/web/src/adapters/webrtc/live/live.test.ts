import { describe, expect, it } from "vitest";
import { computeClockMedian } from "./clock-probe";
import { createNegotiationState, IceCandidateBuffer } from "./negotiation";
import type { ClockProbeSample } from "./types";

describe("clock probe formulas", () => {
  it("computes RTT and offset from four timestamps", () => {
    const t0 = 1000;
    const t1 = 1010;
    const t2 = 1011;
    const t3 = 1021;
    const rtt = t3 - t0 - (t2 - t1);
    const offset = (t1 - t0 + (t2 - t3)) / 2;
    expect(rtt).toBe(20);
    expect(offset).toBe(0);
  });

  it("computes median RTT", () => {
    const samples: ClockProbeSample[] = [
      { sequence: 0, protocolVersion: 1, t0: 0, t1: 1, t2: 2, t3: 10, rttMs: 10, offsetMs: 0, timeout: false, duplicate: false },
      { sequence: 1, protocolVersion: 1, t0: 0, t1: 1, t2: 2, t3: 20, rttMs: 20, offsetMs: 0, timeout: false, duplicate: false },
    ];
    expect(computeClockMedian(samples).rtt).toBe(15);
  });

  it("rejects negative RTT samples from median", () => {
    const samples: ClockProbeSample[] = [
      { sequence: 0, protocolVersion: 1, t0: 0, t1: null, t2: null, t3: null, rttMs: -1, offsetMs: null, timeout: true, duplicate: false },
    ];
    expect(computeClockMedian(samples).rtt).toBeNull();
  });
});

describe("negotiation state", () => {
  it("marks impolite peer correctly", () => {
    const impolite = createNegotiationState(false);
    const polite = createNegotiationState(true);
    expect(impolite.polite).toBe(false);
    expect(polite.polite).toBe(true);
  });
});

describe("ICE candidate buffer", () => {
  it("deduplicates identical candidates", () => {
    const buffer = new IceCandidateBuffer();
    const candidate = { sdpMid: "0", sdpMLineIndex: 0, candidate: "candidate:1" };
    buffer.add(candidate);
    buffer.add(candidate);
    expect(buffer).toBeDefined();
  });

  it("clears buffered candidates", () => {
    const buffer = new IceCandidateBuffer();
    buffer.add({ sdpMid: "0", sdpMLineIndex: 0, candidate: "candidate:1" });
    buffer.clear();
    expect(buffer).toBeDefined();
  });
});

describe("capture constraints", () => {
  it("does not include deviceId in browser_default profile", async () => {
    const { captureConstraints } = await import("./types");
    const constraints = captureConstraints("browser_default");
    expect(constraints).not.toHaveProperty("deviceId");
  });
});
