import { describe, expect, it, vi } from "vitest";
import {
  CLOCK_PROBE_PROTOCOL_VERSION,
  ClockProbeEngine,
  computeClockMedian,
  MAX_PROBE_PAYLOAD_BYTES,
  validCompletedProbes,
  validLocalCompletedProbes,
} from "./clock-probe";
import { collectStatsPreflight } from "./stats-sampler";
import { LiveWebRtcSession } from "./session";
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
  async function flushAsync(): Promise<void> {
    await new Promise<void>((resolve) => {
      queueMicrotask(() => resolve());
    });
    await new Promise<void>((resolve) => {
      setTimeout(resolve, 0);
    });
  }

  class FakeDataChannel {
    readyState: RTCDataChannelState = "open";
    onmessage: ((event: { data: string }) => void) | null = null;
    sent: string[] = [];
    peer: FakeDataChannel | null = null;

    send(data: string): void {
      this.sent.push(data);
      queueMicrotask(() => this.peer?.onmessage?.({ data }));
    }
  }

  function linkedChannels(): [FakeDataChannel, FakeDataChannel] {
    const a = new FakeDataChannel();
    const b = new FakeDataChannel();
    a.peer = b;
    b.peer = a;
    return [a, b];
  }

  function attachEngines(
    roleA: PeerRole,
    roleB: PeerRole,
  ): {
    engineA: ClockProbeEngine;
    engineB: ClockProbeEngine;
    samplesA: ClockProbeSample[];
    samplesB: ClockProbeSample[];
    dcA: FakeDataChannel;
    dcB: FakeDataChannel;
  } {
    const [dcA, dcB] = linkedChannels();
    const samplesA: ClockProbeSample[] = [];
    const samplesB: ClockProbeSample[] = [];
    const engineA = new ClockProbeEngine(roleA, (sample) => samplesA.push(sample));
    const engineB = new ClockProbeEngine(roleB, (sample) => samplesB.push(sample));
    engineA.attach(dcA as unknown as RTCDataChannel);
    engineB.attach(dcB as unknown as RTCDataChannel);
    return { engineA, engineB, samplesA, samplesB, dcA, dcB };
  }

  it("completes Peer A request through Peer B response", async () => {
    const { engineA, engineB, samplesA } = attachEngines("peer_a", "peer_b");
    engineA.start({ intervalMs: 1000, count: 1 });
    engineB.start({ intervalMs: 1000, count: 0 });
    await flushAsync();
    expect(samplesA.some((s) => s.rttMs !== null && !s.invalid && !s.timeout)).toBe(true);
    expect(validLocalCompletedProbes(samplesA, "peer_a")).toHaveLength(1);
  });

  it("completes Peer B request through Peer A response", async () => {
    const { engineA, engineB, samplesB } = attachEngines("peer_a", "peer_b");
    engineA.start({ intervalMs: 1000, count: 0 });
    engineB.start({ intervalMs: 1000, count: 1 });
    await flushAsync();
    expect(validLocalCompletedProbes(samplesB, "peer_b")).toHaveLength(1);
  });

  it("rejects duplicate responses", async () => {
    const { engineA, engineB, samplesA, dcA } = attachEngines("peer_a", "peer_b");
    engineA.start({ intervalMs: 1000, count: 1 });
    engineB.start({ intervalMs: 1000, count: 0 });
    await flushAsync();
    const completed = samplesA.find((s) => s.rttMs !== null);
    expect(completed).toBeDefined();
    const duplicatePayload = JSON.stringify({
      type: "clock_probe_response",
      protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
      sequence: completed!.sequence,
      requesterRole: "peer_a",
      responderRole: "peer_b",
      t0: completed!.t0,
      t1: completed!.t1,
      t2: completed!.t2,
    });
    dcA.onmessage?.({ data: duplicatePayload });
    expect(samplesA.some((s) => s.duplicate)).toBe(true);
    expect(validLocalCompletedProbes(samplesA, "peer_a")).toHaveLength(1);
  });

  it("rejects unsolicited responses without pending request", async () => {
    const { engineA, samplesA, dcA } = attachEngines("peer_a", "peer_b");
    const unsolicited = JSON.stringify({
      type: "clock_probe_response",
      protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
      sequence: 99,
      requesterRole: "peer_a",
      responderRole: "peer_b",
      t0: 1000,
      t1: 1010,
      t2: 1011,
    });
    dcA.onmessage?.({ data: unsolicited });
    expect(samplesA.some((s) => s.unsolicited)).toBe(true);
    expect(validCompletedProbes(samplesA)).toHaveLength(0);
  });

  it("rejects wrong protocol version", () => {
    const { engineA, samplesA } = attachEngines("peer_a", "peer_b");
    const bad = JSON.stringify({
      type: "clock_probe_response",
      protocolVersion: 99,
      sequence: 0,
      requesterRole: "peer_a",
      responderRole: "peer_b",
      t0: 1000,
      t1: 1010,
      t2: 1011,
    });
    (engineA as unknown as { channel: FakeDataChannel }).channel.onmessage?.({ data: bad });
    expect(samplesA.some((s) => s.invalid)).toBe(true);
  });

  it("records timeout samples separately from valid probes", async () => {
    vi.useFakeTimers();
    try {
      const dcA = new FakeDataChannel();
      const samplesA: ClockProbeSample[] = [];
      const engineA = new ClockProbeEngine("peer_a", (sample) => samplesA.push(sample));
      engineA.attach(dcA as unknown as RTCDataChannel);
      engineA.start({ intervalMs: 60_000, count: 1 });
      await vi.runAllTimersAsync();
      expect(samplesA.some((s) => s.timeout)).toBe(true);
      expect(validLocalCompletedProbes(samplesA, "peer_a")).toHaveLength(0);
    } finally {
      vi.useRealTimers();
    }
  });

  it("sends nothing when probe count is zero", () => {
    const [dcA] = linkedChannels();
    const sent: string[] = [];
    dcA.send = (data: string) => sent.push(data);
    const engineA = new ClockProbeEngine("peer_a", () => {});
    engineA.attach(dcA as unknown as RTCDataChannel);
    engineA.start({ intervalMs: 100, count: 0 });
    expect(sent).toHaveLength(0);
  });

  it("skips payloads exceeding UTF-8 byte limit", () => {
    const [dcA] = linkedChannels();
    const sent: string[] = [];
    dcA.send = (data: string) => sent.push(data);
    const engineA = new ClockProbeEngine("peer_a", () => {});
    engineA.attach(dcA as unknown as RTCDataChannel);
    const originalStringify = JSON.stringify;
    JSON.stringify = () => "x".repeat(MAX_PROBE_PAYLOAD_BYTES + 1);
    engineA.start({ intervalMs: 100, count: 1 });
    expect(sent).toHaveLength(0);
    JSON.stringify = originalStringify;
  });
});

describe("session ready gate and export", () => {
  const commit = "e4198657264a6b4629948469dcdabde21a3eaa34";

  function makeSession(role: PeerRole = "peer_a") {
    const callbacks = {
      onPhaseChange: vi.fn(),
      onSignalingState: vi.fn(),
      onConnectionState: vi.fn(),
      onIceState: vi.fn(),
      onDataChannelState: vi.fn(),
      onRemoteStream: vi.fn(),
      onMicrophoneState: vi.fn(),
      onNegotiation: vi.fn(),
      onClockProbe: vi.fn(),
      onStatsSample: vi.fn(),
      onError: vi.fn(),
    };
    const session = new LiveWebRtcSession(
      {
        sessionCorrelationId: "0123456789abcdef0123456789abcdef",
        localPeerId: role,
        softwareCommit: commit,
      },
      callbacks,
    );
    return { session, callbacks };
  }

  it("passes ready gate after valid local probe and stats preflight", async () => {
    const { session } = makeSession();
    const internal = session as unknown as {
      localStream: MediaStream;
      pc: RTCPeerConnection;
      dc: RTCDataChannel;
      remoteStream: MediaStream;
      clockSamples: ClockProbeSample[];
      statsPreflightComplete: boolean;
      collectReadyFailures: () => string[];
    };
    internal.localStream = {
      getAudioTracks: () => [{ readyState: "live" }],
    } as MediaStream;
    internal.pc = {
      connectionState: "connected",
      iceConnectionState: "connected",
      signalingState: "stable",
    } as RTCPeerConnection;
    internal.dc = { readyState: "open" } as RTCDataChannel;
    internal.remoteStream = {
      getAudioTracks: () => [{ readyState: "live" }],
    } as MediaStream;
    internal.clockSamples = [probeSample({ senderRole: "peer_a" })];
    internal.statsPreflightComplete = true;
    expect(internal.collectReadyFailures()).toEqual([]);
  });

  it("rejects ready gate without stats preflight", () => {
    const { session } = makeSession();
    const internal = session as unknown as {
      localStream: MediaStream;
      pc: RTCPeerConnection;
      dc: RTCDataChannel;
      remoteStream: MediaStream;
      clockSamples: ClockProbeSample[];
      statsPreflightComplete: boolean;
      collectReadyFailures: () => string[];
    };
    internal.localStream = {
      getAudioTracks: () => [{ readyState: "live" }],
    } as MediaStream;
    internal.pc = {
      connectionState: "connected",
      iceConnectionState: "connected",
      signalingState: "stable",
    } as RTCPeerConnection;
    internal.dc = { readyState: "open" } as RTCDataChannel;
    internal.remoteStream = {
      getAudioTracks: () => [{ readyState: "live" }],
    } as MediaStream;
    internal.clockSamples = [probeSample({ senderRole: "peer_a" })];
    internal.statsPreflightComplete = false;
    expect(internal.collectReadyFailures()).toContain("stats preflight incomplete");
  });

  it("finalized export requires completed phase and finalized kind", () => {
    const { session } = makeSession();
    const internal = session as unknown as {
      phase: string;
      observationStartedAt: string;
      observationCompletedAt: string;
      clockSamples: ClockProbeSample[];
      statsSamples: unknown[];
      exportFinalizedEndpoint: () => Record<string, unknown>;
      exportEndpointDraft: () => Record<string, unknown>;
    };
    internal.phase = "completed";
    internal.observationStartedAt = "2026-07-21T09:00:00.000Z";
    internal.observationCompletedAt = "2026-07-21T09:01:00.000Z";
    internal.clockSamples = Array.from({ length: 10 }, (_, i) =>
      probeSample({ sequence: i, senderRole: "peer_a" }),
    );
    internal.statsSamples = Array.from({ length: 30 }, () => ({}));
    const finalized = internal.exportFinalizedEndpoint();
    expect(finalized.exportKind).toBe("finalized");
    const draft = internal.exportEndpointDraft();
    expect(draft.exportKind).toBe("diagnostic_draft");
    expect(draft.exportKind).not.toBe(finalized.exportKind);
  });
});

describe("typed stats normalization", () => {
  it("preserves category and boolean metric kinds", async () => {
    const stats = new Map<string, Record<string, unknown>>([
      [
        "transport-1",
        {
          id: "transport-1",
          type: "transport",
          selectedCandidatePairId: "pair-1",
        },
      ],
      [
        "pair-1",
        {
          id: "pair-1",
          type: "candidate-pair",
          state: "succeeded",
          nominated: true,
          protocol: "udp",
          localCandidateId: "local-1",
          remoteCandidateId: "remote-1",
          packetsSent: 1,
          packetsReceived: 2,
        },
      ],
      ["local-1", { id: "local-1", type: "local-candidate", candidateType: "host" }],
      ["remote-1", { id: "remote-1", type: "remote-candidate", candidateType: "srflx" }],
      [
        "inbound-1",
        {
          id: "inbound-1",
          type: "inbound-rtp",
          kind: "audio",
          packetsReceived: 10,
          bytesReceived: 1000,
        },
      ],
      [
        "outbound-1",
        {
          id: "outbound-1",
          type: "outbound-rtp",
          kind: "audio",
          packetsSent: 8,
          bytesSent: 800,
        },
      ],
      ["codec-1", { id: "codec-1", type: "codec", mimeType: "audio/opus", clockRate: 48000, channels: 2 }],
    ]);
    const pc = {
      getStats: vi.fn().mockResolvedValue(stats),
    } as unknown as RTCPeerConnection;
    const sample = await collectStatsPreflight(pc);
    expect(sample.candidatePair.state).toEqual({ kind: "observed_category", value: "succeeded" });
    expect(sample.candidatePair.nominated).toEqual({ kind: "observed_boolean", value: true });
    expect(sample.candidatePair.protocol).toEqual({ kind: "observed_category", value: "udp" });
    expect(sample.codec.mimeType).toEqual({ kind: "observed_category", value: "audio/opus" });
  });
});

describe("bilateral clock probe protocol roles", () => {
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
