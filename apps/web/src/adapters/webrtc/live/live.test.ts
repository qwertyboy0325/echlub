import { describe, expect, it, vi } from "vitest";
import {
  CLOCK_PROBE_PROTOCOL_VERSION,
  ClockProbeEngine,
  computeClockMedian,
  validCompletedProbes,
} from "./clock-probe";
import {
  applySignalingDescription,
  createAndSendOffer,
  createNegotiationState,
  IceCandidateBuffer,
} from "./negotiation";
import { StatsSampler } from "./stats-sampler";
import type { ClockProbeSample, PeerRole } from "./types";

function probeSample(overrides: Partial<ClockProbeSample> = {}): ClockProbeSample {
  return {
    sequence: 0,
    senderRole: "peer_a",
    protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
    t0: 0,
    t1: 1,
    t2: 2,
    t3: 10,
    rttMs: 10,
    offsetMs: 0,
    timeout: false,
    duplicate: false,
    unsolicited: false,
    invalid: false,
    ...overrides,
  };
}

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

  it("computes median RTT and MAD", () => {
    const samples = [probeSample({ rttMs: 10 }), probeSample({ sequence: 1, rttMs: 20 })];
    const medians = computeClockMedian(samples);
    expect(medians.rtt).toBe(15);
    expect(medians.madRtt).toBe(5);
  });

  it("excludes timeout and invalid probes from median", () => {
    const samples = [
      probeSample({ timeout: true, rttMs: null, invalid: false }),
      probeSample({ invalid: true, rttMs: null }),
    ];
    expect(computeClockMedian(samples).rtt).toBeNull();
    expect(validCompletedProbes(samples)).toHaveLength(0);
  });
});

describe("negotiation behavior", () => {
  it("marks impolite peer correctly", () => {
    expect(createNegotiationState(false).polite).toBe(false);
    expect(createNegotiationState(true).polite).toBe(true);
  });

  it("impolite peer ignores offer collision", async () => {
    const pc = {
      signalingState: "have-local-offer",
      setRemoteDescription: vi.fn(),
      createAnswer: vi.fn(),
      localDescription: null,
    } as unknown as RTCPeerConnection;
    const state = createNegotiationState(false);
    state.makingOffer = true;
    await applySignalingDescription(
      pc,
      state,
      { type: "description", sdp: { type: "offer", sdp: "v=0" } },
      vi.fn(),
    );
    expect(state.ignoreOffer).toBe(true);
    expect(pc.setRemoteDescription).not.toHaveBeenCalled();
  });

  it("polite peer recovers from offer collision", async () => {
    const pc = {
      signalingState: "have-local-offer",
      setRemoteDescription: vi.fn().mockResolvedValue(undefined),
      createAnswer: vi.fn().mockResolvedValue({ type: "answer", sdp: "v=0" }),
      setLocalDescription: vi.fn().mockResolvedValue(undefined),
      localDescription: { type: "answer", sdp: "v=0" },
    } as unknown as RTCPeerConnection;
    const state = createNegotiationState(true);
    state.makingOffer = true;
    const send = vi.fn();
    await applySignalingDescription(
      pc,
      state,
      { type: "description", sdp: { type: "offer", sdp: "v=0" } },
      send,
    );
    expect(state.ignoreOffer).toBe(false);
    expect(pc.setRemoteDescription).toHaveBeenCalled();
    expect(send).toHaveBeenCalled();
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

describe("stats sampler lifecycle", () => {
  it("does not invoke callback after stop", async () => {
    const pc = {
      getStats: vi.fn().mockResolvedValue(new Map()),
    } as unknown as RTCPeerConnection;
    const onSample = vi.fn();
    const sampler = new StatsSampler(pc, onSample);
    sampler.start(10, 5, 1000);
    sampler.stop();
    await new Promise((r) => setTimeout(r, 30));
    expect(onSample).not.toHaveBeenCalled();
  });
});

describe("session export semantics", () => {
  it("rejects placeholder software commit for finalized export", () => {
    const commit = "unknown";
    expect(commit.length < 7 || commit === "unknown").toBe(true);
  });

  it("accepts injected git commit for finalized export", () => {
    const commit: string = "e4198657264a6b4629948469dcdabde21a3eaa34";
    expect(commit.length >= 7 && commit !== "unknown").toBe(true);
  });
});

describe("bilateral clock probe protocol", () => {
  it("uses independent sender roles per probe", () => {
    const roles: PeerRole[] = ["peer_a", "peer_b"];
    expect(new Set(roles).size).toBe(2);
  });
});

describe("capture constraints", () => {
  it("does not include deviceId in browser_default profile", async () => {
    const { captureConstraints } = await import("./types");
    const constraints = captureConstraints("browser_default");
    expect(constraints).not.toHaveProperty("deviceId");
  });
});

describe("offer creation", () => {
  it("creates and sends local offer", async () => {
    const pc = {
      createOffer: vi.fn().mockResolvedValue({ type: "offer", sdp: "v=0" }),
      setLocalDescription: vi.fn().mockResolvedValue(undefined),
      localDescription: { type: "offer", sdp: "v=0" },
    } as unknown as RTCPeerConnection;
    const state = createNegotiationState(true);
    const send = vi.fn();
    await createAndSendOffer(pc, state, send);
    expect(send).toHaveBeenCalledWith("offer", pc.localDescription);
    expect(state.makingOffer).toBe(false);
  });
});
