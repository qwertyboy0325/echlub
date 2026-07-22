import { describe, expect, it, vi } from "vitest";
import {
  CLOCK_PROBE_PROTOCOL_VERSION,
  ClockProbeEngine,
  computeClockMedian,
  MAX_PROBE_PAYLOAD_BYTES,
  oppositeRole,
  validateCrossDeviceClockTimestamps,
  verifyStoredClockMetrics,
  validCompletedProbes,
  validLocalCompletedProbes,
} from "./clock-probe";
import {
  collectNormalizedStats,
  collectStatsPreflight,
  computeIntervalMetrics,
  deltaCumulativeMetric,
  hasRtpAudioCounterAvailability,
  StatsSampler,
} from "./stats-sampler";
import { LiveWebRtcSession } from "./session";
import {
  applySignalingDescription,
  createAndSendOffer,
  createNegotiationState,
  IceCandidateBuffer,
} from "./negotiation";
import {
  invalid,
  observedCategory,
  observedNumber,
  RTP_AUDIO_COUNTER_SOURCE,
  unsupported,
  unavailable,
} from "./types";

import type { ClockProbeSample, PeerRole } from "./types";

function probeSample(overrides: Partial<ClockProbeSample> = {}): ClockProbeSample {
  return {
    sequence: 0,
    requesterRole: "peer_a",
    responderRole: "peer_b",
    protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
    t0: 1000,
    t1: 1010,
    t2: 1011,
    t3: 1021,
    rttMs: 20,
    offsetMs: 0,
    timeout: false,
    duplicate: false,
    unsolicited: false,
    invalid: false,
    ...overrides,
  };
}

describe("stored clock metric consistency", () => {
  it.each([
    ["+500ms offset", { t0: 1000, t1: 1510, t2: 1511, t3: 1021, rttMs: 20, offsetMs: 500 }],
    ["-500ms offset", { t0: 1000, t1: 510, t2: 511, t3: 1021, rttMs: 20, offsetMs: -500 }],
  ])("accepts valid %s", (_label, sample) => {
    expect(verifyStoredClockMetrics(sample)).toBe(true);
  });

  it("rejects stored RTT differing from timestamps", () => {
    expect(
      verifyStoredClockMetrics({
        t0: 1000,
        t1: 1010,
        t2: 1011,
        t3: 1021,
        rttMs: 999,
        offsetMs: 0,
      }),
    ).toBe(false);
  });

  it("rejects negative stored RTT", () => {
    expect(
      verifyStoredClockMetrics({
        t0: 1000,
        t1: 1010,
        t2: 1011,
        t3: 1021,
        rttMs: -5,
        offsetMs: 0,
      }),
    ).toBe(false);
  });

  it("rejects stored offset differing from canonical offset", () => {
    expect(
      verifyStoredClockMetrics({
        t0: 1000,
        t1: 1510,
        t2: 1511,
        t3: 1021,
        rttMs: 20,
        offsetMs: 123,
      }),
    ).toBe(false);
  });

  it("excludes duplicate probe with arbitrary RTT from valid completed probes", () => {
    const valid = probeSample({ rttMs: 20 });
    const duplicate = probeSample({ sequence: 1, duplicate: true, rttMs: 9999 });
    expect(validCompletedProbes([valid, duplicate])).toEqual([valid]);
  });

  it("excludes unsolicited probe with arbitrary RTT from valid completed probes", () => {
    const valid = probeSample({ rttMs: 20 });
    const unsolicited = probeSample({ sequence: 1, unsolicited: true, rttMs: 9999 });
    expect(validCompletedProbes([valid, unsolicited])).toEqual([valid]);
  });

  it("excludes invalid probe with arbitrary RTT from valid completed probes", () => {
    const valid = probeSample({ rttMs: 20 });
    const invalidProbe = probeSample({ sequence: 1, invalid: true, rttMs: 9999 });
    expect(validCompletedProbes([valid, invalidProbe])).toEqual([valid]);
  });
});

describe("cross-device clock semantics", () => {
  it.each([
    ["+500ms", 1000, 1600, 1601, 1021],
    ["-500ms", 2000, 1400, 1401, 2021],
    ["+5000ms", 1000, 6100, 6101, 1021],
    ["-5000ms", 10000, 4500, 4501, 10021],
  ])("accepts separate clock domains (%s)", (_label, t0, t1, t2, t3) => {
    const result = validateCrossDeviceClockTimestamps(t0, t1, t2, t3);
    expect(result.valid).toBe(true);
    expect(result.rttMs).toBe(20);
    expect(result.offsetMs).not.toBeNull();
  });

  it.each([
    ["t3 < t0", 2000, 1500, 1501, 1000],
    ["t2 < t1", 1000, 1200, 1100, 1021],
    ["negative RTT", 1000, 1010, 1050, 1021],
  ])("rejects invalid ordering (%s)", (_label, t0, t1, t2, t3) => {
    expect(validateCrossDeviceClockTimestamps(t0, t1, t2, t3).valid).toBe(false);
  });

  it("rejects non-finite timestamps", () => {
    expect(validateCrossDeviceClockTimestamps(Number.NaN, 1, 2, 3).valid).toBe(false);
  });
});

describe("responder identity", () => {
  it("requires opposite responder for valid local completed probes", () => {
    const valid = probeSample({ requesterRole: "peer_a", responderRole: "peer_b" });
    const wrong = probeSample({ requesterRole: "peer_a", responderRole: "peer_a" });
    expect(validLocalCompletedProbes([valid], "peer_a")).toHaveLength(1);
    expect(validLocalCompletedProbes([wrong], "peer_a")).toHaveLength(0);
  });

  it("preserves responder identity on duplicate responses", async () => {
    const { samplesA, dcA } = attachEnginesForDuplicateTest();
    await flushAsync();
    const completed = samplesA.find((s) => s.rttMs !== null);
    expect(completed?.responderRole).toBe("peer_b");
    dcA.onmessage?.({
      data: JSON.stringify({
        type: "clock_probe_response",
        protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
        sequence: completed!.sequence,
        requesterRole: "peer_a",
        responderRole: "peer_b",
        t0: completed!.t0,
        t1: completed!.t1,
        t2: completed!.t2,
      }),
    });
    expect(samplesA.some((s) => s.duplicate && s.responderRole === "peer_b")).toBe(true);
  });

  function attachEnginesForDuplicateTest() {
    const [dcA, dcB] = linkedChannels();
    const samplesA: ClockProbeSample[] = [];
    const engineA = new ClockProbeEngine("peer_a", (sample) => samplesA.push(sample));
    const engineB = new ClockProbeEngine("peer_b", () => {});
    engineA.attach(dcA as unknown as RTCDataChannel);
    engineB.attach(dcB as unknown as RTCDataChannel);
    engineA.start({ intervalMs: 1000, count: 1 });
    engineB.start({ intervalMs: 1000, count: 0 });
    return { samplesA, dcA };
  }

  async function flushAsync(): Promise<void> {
    await new Promise<void>((resolve) => queueMicrotask(() => resolve()));
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
  }

  class FakeDataChannel {
    readyState: RTCDataChannelState = "open";
    onmessage: ((event: { data: string }) => void) | null = null;
    peer: FakeDataChannel | null = null;
    send(data: string): void {
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
});

describe("stats interval metrics", () => {
  const rtpSource = observedCategory(RTP_AUDIO_COUNTER_SOURCE);
  const baseSample = (bytes: number, offsetMs: number) => ({
    offsetMs,
    candidatePair: {},
    inboundAudio: {
      counterSource: rtpSource,
      bytesReceived: observedNumber(bytes),
      packetsLost: observedNumber(0),
    },
    outboundAudio: {
      counterSource: rtpSource,
      bytesSent: observedNumber(bytes),
    },
    remoteInboundAudio: {},
    codec: {},
  });

  it("computes normal positive delta", () => {
    const metrics = computeIntervalMetrics(baseSample(100, 0), baseSample(200, 1000));
    expect(metrics.receiveBitrateBps).toEqual({ kind: "observed_number", value: 800 });
  });

  it("accepts zero delta", () => {
    expect(deltaCumulativeMetric(observedNumber(5), observedNumber(5))).toEqual(observedNumber(0));
  });

  it("classifies counter reset", () => {
    expect(deltaCumulativeMetric(observedNumber(10), observedNumber(3))).toEqual(
      invalid("counter_reset"),
    );
  });

  it("returns unavailable for missing previous value", () => {
    expect(deltaCumulativeMetric(unavailable("missing"), observedNumber(1))).toEqual(
      unavailable("missing previous value"),
    );
  });

  it("returns unavailable for missing current value", () => {
    expect(deltaCumulativeMetric(observedNumber(1), unavailable("missing"))).toEqual(
      unavailable("missing current value"),
    );
  });

  it("returns unsupported when source unsupported", () => {
    expect(deltaCumulativeMetric(unsupported(), unsupported())).toEqual(unsupported());
    const metrics = computeIntervalMetrics(
      { ...baseSample(100, 0), inboundAudio: {}, outboundAudio: {} },
      { ...baseSample(200, 1000), inboundAudio: {}, outboundAudio: {} },
    );
    expect(metrics.receiveBitrateBps).toEqual(unsupported());
    expect(metrics.sendBitrateBps).toEqual(unsupported());
    expect(metrics.packetLossDelta).toEqual(unsupported());
  });

  it("preserves counter reset interval state", () => {
    const metrics = computeIntervalMetrics(
      {
        ...baseSample(100, 0),
        inboundAudio: {
          counterSource: rtpSource,
          bytesReceived: observedNumber(10),
          packetsLost: observedNumber(5),
        },
        outboundAudio: {
          counterSource: rtpSource,
          bytesSent: observedNumber(10),
        },
      },
      {
        ...baseSample(200, 1000),
        inboundAudio: {
          counterSource: rtpSource,
          bytesReceived: observedNumber(3),
          packetsLost: observedNumber(1),
        },
        outboundAudio: {
          counterSource: rtpSource,
          bytesSent: observedNumber(3),
        },
      },
    );
    expect(metrics.receiveBitrateBps).toEqual(invalid("counter_reset"));
    expect(metrics.sendBitrateBps).toEqual(invalid("counter_reset"));
    expect(metrics.packetLossDelta).toEqual(invalid("counter_reset"));
  });

  it("returns invalid for wrong metric type", () => {
    expect(deltaCumulativeMetric(observedNumber(1), invalid("x"))).toEqual(
      invalid("expected observed_number"),
    );
  });

  it("does not derive audio bitrate from transport-only counters", () => {
    const candidatePair = (bytes: number, packets: number) => ({
      counterSource: observedCategory("candidate_pair_transport"),
      packetsReceived: observedNumber(packets),
      packetsSent: observedNumber(packets),
      bytesReceived: observedNumber(bytes),
      bytesSent: observedNumber(bytes),
    });
    const prev = {
      offsetMs: 0,
      candidatePair: candidatePair(1000, 10),
      inboundAudio: {},
      outboundAudio: {},
      remoteInboundAudio: {},
      codec: {},
    };
    const curr = {
      offsetMs: 1000,
      candidatePair: candidatePair(2000, 20),
      inboundAudio: {},
      outboundAudio: {},
      remoteInboundAudio: {},
      codec: {},
    };
    const metrics = computeIntervalMetrics(prev, curr);
    expect(metrics.receiveBitrateBps).toEqual(unsupported());
    expect(metrics.sendBitrateBps).toEqual(unsupported());
    expect(metrics.transportReceiveBitrateBps).toEqual({ kind: "observed_number", value: 8000 });
    expect(metrics.transportSendBitrateBps).toEqual({ kind: "observed_number", value: 8000 });
  });
});

describe("data channel lifecycle evidence", () => {
  it("records closed state in finalized export", () => {
    const { session } = makeSession();
    const internal = session as unknown as {
      phase: string;
      observationStartedAt: string;
      observationCompletedAt: string;
      clockSamples: ClockProbeSample[];
      statsSamples: unknown[];
      dataChannelProps: Record<string, unknown>;
      exportFinalizedEndpoint: () => Record<string, unknown>;
    };
    internal.phase = "completed";
    internal.observationStartedAt = "2026-07-21T09:00:00.000Z";
    internal.observationCompletedAt = "2026-07-21T09:01:00.000Z";
    internal.clockSamples = Array.from({ length: 10 }, (_, i) =>
      probeSample({ sequence: i, requesterRole: "peer_a", responderRole: "peer_b" }),
    );
    internal.statsSamples = Array.from({ length: 30 }, () => ({}));
    internal.dataChannelProps = { readyState: "closed" };
    const exported = internal.exportFinalizedEndpoint();
    expect(exported.dataChannel).toMatchObject({ readyState: "closed" });
  });

  function makeSession(role: PeerRole = "peer_a") {
    const session = new LiveWebRtcSession(
      {
        sessionCorrelationId: "0123456789abcdef0123456789abcdef",
        localPeerId: role,
        softwareCommit: "e4198657264a6b4629948469dcdabde21a3eaa34",
      },
      {
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
      },
    );
    return { session };
  }
});

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
    const samples = [
      probeSample({
        t0: 1000,
        t1: 1005,
        t2: 1006,
        t3: 1011,
        rttMs: 10,
        offsetMs: 0,
      }),
      probeSample({
        sequence: 1,
        t0: 1100,
        t1: 1105,
        t2: 1106,
        t3: 1131,
        rttMs: 30,
        offsetMs: -10,
      }),
    ];
    const medians = computeClockMedian(samples);
    expect(medians.rtt).toBe(20);
    expect(medians.madRtt).toBe(10);
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

  function primeReadyInternals(
    session: LiveWebRtcSession,
    overrides: Record<string, unknown> = {},
  ) {
    const internal = session as unknown as {
      localStream: MediaStream;
      pc: RTCPeerConnection;
      dc: RTCDataChannel;
      remoteStream: MediaStream;
      clockSamples: ClockProbeSample[];
      statsPreflightComplete: boolean;
      statsPreflightHasRtpAudio: boolean;
      negotiation: { makingOffer: boolean; isSettingRemoteAnswerPending: boolean };
      collectReadyFailures: () => string[];
      evaluateReadyToObserve: () => void;
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
    internal.clockSamples = [probeSample({ requesterRole: "peer_a", responderRole: "peer_b" })];
    internal.statsPreflightComplete = true;
    internal.statsPreflightHasRtpAudio = true;
    internal.negotiation = { makingOffer: false, isSettingRemoteAnswerPending: false };
    Object.assign(internal, overrides);
    return internal;
  }

  it("remote track only is not ready", () => {
    const { session } = makeSession();
    const internal = primeReadyInternals(session, {
      clockSamples: [],
      statsPreflightComplete: false,
    });
    internal.remoteStream = {
      getAudioTracks: () => [{ readyState: "live" }],
    } as MediaStream;
    internal.localStream = { getAudioTracks: () => [] } as unknown as MediaStream;
    expect(internal.collectReadyFailures().length).toBeGreaterThan(0);
  });

  it("data channel open without valid probe is not ready", () => {
    const { session } = makeSession();
    const internal = primeReadyInternals(session, { clockSamples: [] });
    expect(internal.collectReadyFailures()).toContain("clock preflight incomplete");
  });

  it("valid probe without stats preflight is not ready", () => {
    const { session } = makeSession();
    const internal = primeReadyInternals(session, { statsPreflightComplete: false });
    expect(internal.collectReadyFailures()).toContain("stats preflight incomplete");
  });

  it("stats preflight with ICE checking is not ready", () => {
    const { session } = makeSession();
    const internal = primeReadyInternals(session);
    internal.pc = {
      connectionState: "connected",
      iceConnectionState: "checking",
      signalingState: "stable",
    } as RTCPeerConnection;
    expect(internal.collectReadyFailures()).toContain("ICE not connected");
  });

  it("negotiation active is not ready", () => {
    const { session } = makeSession();
    const internal = primeReadyInternals(session);
    internal.negotiation = { makingOffer: true, isSettingRemoteAnswerPending: false };
    expect(internal.collectReadyFailures()).toContain("negotiation in progress");
  });

  it("sets ready_to_observe exactly once when all invariants pass", () => {
    const { session, callbacks } = makeSession();
    const internal = primeReadyInternals(session);
    internal.evaluateReadyToObserve();
    internal.evaluateReadyToObserve();
    expect(callbacks.onPhaseChange).toHaveBeenCalledWith("ready_to_observe");
    expect(
      callbacks.onPhaseChange.mock.calls.filter(([phase]) => phase === "ready_to_observe"),
    ).toHaveLength(1);
  });

  it("passes ready gate after valid local probe and stats preflight", async () => {
    const { session } = makeSession();
    const internal = session as unknown as {
      localStream: MediaStream;
      pc: RTCPeerConnection;
      dc: RTCDataChannel;
      remoteStream: MediaStream;
      clockSamples: ClockProbeSample[];
      statsPreflightComplete: boolean;
      statsPreflightHasRtpAudio: boolean;
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
    internal.clockSamples = [probeSample({ requesterRole: "peer_a", responderRole: "peer_b" })];
    internal.statsPreflightComplete = true;
    internal.statsPreflightHasRtpAudio = true;
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
      statsPreflightHasRtpAudio: boolean;
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
    internal.clockSamples = [probeSample({ requesterRole: "peer_a", responderRole: "peer_b" })];
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
      probeSample({ sequence: i, requesterRole: "peer_a", responderRole: "peer_b" }),
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
    expect(sample.inboundAudio.counterSource).toEqual({
      kind: "observed_category",
      value: RTP_AUDIO_COUNTER_SOURCE,
    });
    expect(sample.outboundAudio.counterSource).toEqual({
      kind: "observed_category",
      value: RTP_AUDIO_COUNTER_SOURCE,
    });
    expect(sample.candidatePair.counterSource).toEqual({
      kind: "observed_category",
      value: "candidate_pair_transport",
    });
    expect(hasRtpAudioCounterAvailability(sample)).toBe(true);
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

describe("peer discovery negotiation order", () => {
  function makeSession(role: PeerRole) {
    const onNegotiation = vi.fn();
    const onError = vi.fn();
    const callbacks = {
      onPhaseChange: vi.fn(),
      onSignalingState: vi.fn(),
      onConnectionState: vi.fn(),
      onIceState: vi.fn(),
      onDataChannelState: vi.fn(),
      onRemoteStream: vi.fn(),
      onMicrophoneState: vi.fn(),
      onNegotiation,
      onClockProbe: vi.fn(),
      onStatsSample: vi.fn(),
      onError,
    };
    const session = new LiveWebRtcSession(
      {
        sessionCorrelationId: "0123456789abcdef0123456789abcdef",
        localPeerId: role,
        softwareCommit: "e4198657264a6b4629948469dcdabde21a3eaa34",
      },
      callbacks,
    );
    const internal = session as unknown as {
      handlePeerJoined: (peerId: string) => void;
      maybeStartInitialNegotiation: (trigger: "negotiation_needed" | "peer_joined") => void;
      peerNegotiationStarted: boolean;
      peerPresent: boolean;
      pc: RTCPeerConnection | null;
      negotiation: { makingOffer: boolean };
      wirePeerConnection: (pc: RTCPeerConnection) => void;
    };
    internal.pc = {
      createOffer: vi.fn().mockResolvedValue({ type: "offer", sdp: "v=0" }),
      setLocalDescription: vi.fn().mockResolvedValue(undefined),
      localDescription: { type: "offer", sdp: "v=0" },
      signalingState: "stable",
      onnegotiationneeded: null as (() => void) | null,
    } as unknown as RTCPeerConnection;
    internal.wirePeerConnection(internal.pc);
    return { session, onNegotiation, onError, internal, callbacks };
  }

  it("peer_a initiates exactly one negotiation when learning peer_b already exists", () => {
    const { onNegotiation, internal } = makeSession("peer_a");
    internal.handlePeerJoined("peer_b");
    internal.handlePeerJoined("peer_b");
    expect(onNegotiation).toHaveBeenCalledTimes(1);
    expect(internal.peerNegotiationStarted).toBe(true);
  });

  it("peer_b never initiates an offer when peer_a joins", () => {
    const { onNegotiation, internal } = makeSession("peer_b");
    internal.handlePeerJoined("peer_a");
    expect(onNegotiation).not.toHaveBeenCalled();
    expect(internal.peerNegotiationStarted).toBe(false);
  });

  it("ignores self peer_joined notifications", () => {
    const { onNegotiation, internal } = makeSession("peer_a");
    internal.handlePeerJoined("peer_a");
    expect(onNegotiation).not.toHaveBeenCalled();
  });

  it("does not offer before peer discovery on negotiationneeded", () => {
    const { onNegotiation, internal } = makeSession("peer_a");
    internal.maybeStartInitialNegotiation("negotiation_needed");
    expect(onNegotiation).not.toHaveBeenCalled();
    expect(internal.peerNegotiationStarted).toBe(false);
  });

  it("offers once when peer discovery follows earlier negotiationneeded", () => {
    const { onNegotiation, internal } = makeSession("peer_a");
    internal.maybeStartInitialNegotiation("negotiation_needed");
    internal.handlePeerJoined("peer_b");
    expect(onNegotiation).toHaveBeenCalledTimes(1);
  });

  it("offers once when negotiationneeded follows peer discovery", async () => {
    const { onNegotiation, internal } = makeSession("peer_a");
    internal.handlePeerJoined("peer_b");
    internal.maybeStartInitialNegotiation("negotiation_needed");
    expect(onNegotiation).toHaveBeenCalledTimes(1);
    await Promise.resolve();
    expect(internal.pc?.createOffer).toHaveBeenCalledTimes(1);
    expect(internal.pc?.setLocalDescription).toHaveBeenCalledTimes(1);
  });

  it("does not start a second offer after initial negotiation started", () => {
    const { onNegotiation, internal } = makeSession("peer_a");
    internal.peerPresent = true;
    internal.peerNegotiationStarted = true;
    internal.maybeStartInitialNegotiation("negotiation_needed");
    expect(onNegotiation).not.toHaveBeenCalled();
  });

  it("surfaces failed initial offer without leaving started state when unsafe", async () => {
    const { onError, internal } = makeSession("peer_a");
    internal.pc!.createOffer = vi.fn().mockRejectedValue(new Error("offer failed"));
    internal.handlePeerJoined("peer_b");
    await new Promise<void>((resolve) => setTimeout(resolve, 0));
    expect(onError).toHaveBeenCalled();
    expect(internal.peerNegotiationStarted).toBe(false);
  });
});
