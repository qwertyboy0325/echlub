import {
  observed,
  unavailable,
  unsupported,
  type PerformanceRunDraft,
} from "../../../shared/performance/types";
import {
  correlatePulseSequence,
  computeDetectionStats,
  DEFAULT_PULSE_DETECTION_CONFIG,
  dominantFrequencyHz,
  bandEnergyNearFrequency,
  type PulseDetectionRecord,
  type PulseEmitRecord,
} from "./pulse-detector";

export type HarnessOutcome = "pass" | "harness_limitation" | "connection_failed";

export interface SyntheticRunResult {
  outcome: HarnessOutcome;
  limitationReason?: string;
  observationStartedAt: string;
  observationCompletedAt: string;
  runId: string;
  timing: {
    pulseEmitMs: number | null;
    pulseDetectMs: number | null;
    loopbackLatencyMs: number | null;
    datachannelRttMs: number | null;
    iceGatheringMs: number | null;
    connectionSetupMs: number | null;
    observationWindowMs: number | null;
  };
  syntheticPulse: {
    pulsesEmitted: number;
    pulsesDetected: number;
    detectionRate: number;
    meanDetectionLatencyMs: number | null;
    jitterMs: number | null;
    correlatedPairs: number;
    pulseRecords: Array<{
      index: number;
      emitAtMs: number;
      detectedAtMs: number | null;
      latencyMs: number | null;
      frequencyHz: number;
    }>;
  };
  transport: {
    bytesSent: number;
    bytesReceived: number;
    packetsLost: number;
  };
  mediaPathLive: boolean;
  decodedPulseDetectorUsed: boolean;
}

export interface SyntheticLoopbackOptions {
  runId?: string;
  connectionTimeoutMs?: number;
}

const PULSE_COUNT = 5;
const PULSE_INTERVAL_MS = 200;
const PULSE_FREQ = 1000;
const PULSE_DURATION_S = 0.1;
const INBOUND_WAIT_MS = 5000;
const ONTACK_WAIT_MS = 5000;
const CONNECTION_TIMEOUT_MS = 8000;
const DETECTOR_WARMUP_MS = 300;
const RTT_TIMEOUT_MS = 3000;

export async function runSyntheticLoopback(
  options: SyntheticLoopbackOptions = {},
): Promise<SyntheticRunResult> {
  const runId = options.runId ?? `synthetic-${Date.now()}`;
  const connectionTimeoutMs = options.connectionTimeoutMs ?? CONNECTION_TIMEOUT_MS;
  const observationStartedAt = new Date().toISOString();
  const observationStartMs = performance.now();

  const ctx = new AudioContext();
  await ctx.resume();

  const dest = ctx.createMediaStreamDestination();
  const pcA = new RTCPeerConnection({ iceServers: [] });
  const pcB = new RTCPeerConnection({ iceServers: [] });

  pcA.onicecandidate = (e) => {
    if (e.candidate) void pcB.addIceCandidate(e.candidate);
  };
  pcB.onicecandidate = (e) => {
    if (e.candidate) void pcA.addIceCandidate(e.candidate);
  };

  const setupStart = performance.now();
  const iceStart = performance.now();

  const stream = dest.stream;
  for (const track of stream.getTracks()) {
    pcA.addTrack(track, stream);
  }

  const remoteTrackReady = waitForRemoteTrack(pcB, ONTACK_WAIT_MS);
  const dcA = pcA.createDataChannel("probe");

  const dcRttPromise = waitForDataChannelRtt(dcA, pcB, RTT_TIMEOUT_MS);

  const offer = await pcA.createOffer();
  await pcA.setLocalDescription(offer);
  await pcB.setRemoteDescription(offer);
  const answer = await pcB.createAnswer();
  await pcB.setLocalDescription(answer);
  await pcA.setRemoteDescription(answer);

  const connected = await waitForConnection(pcA, connectionTimeoutMs);
  if (!connected) {
    pcA.close();
    pcB.close();
    await ctx.close();
    return buildFailureResult(
      runId,
      observationStartedAt,
      new Date().toISOString(),
      observationStartMs,
      "connection_failed",
      "peer connection did not reach connected state within timeout",
    );
  }

  const iceGatheringMs = performance.now() - iceStart;
  const connectionSetupMs = performance.now() - setupStart;
  const dcRtt = await dcRttPromise;

  let remoteStream = await remoteTrackReady;
  remoteStream ??= receiverAudioStream(pcB);
  await waitForInboundBytes(pcB, INBOUND_WAIT_MS);

  const rawDetections: PulseDetectionRecord[] = [];
  let stopDetector = () => {};
  if (remoteStream) {
    stopDetector = startDecodedMediaDetector(ctx, remoteStream, PULSE_FREQ, (record) => {
      rawDetections.push(record);
    });
    await sleep(DETECTOR_WARMUP_MS);
  }

  const pulseEmitTimes: PulseEmitRecord[] = [];
  for (let i = 0; i < PULSE_COUNT; i++) {
    const emitAtMs = performance.now();
    pulseEmitTimes.push({ index: i, emitAtMs, frequencyHz: PULSE_FREQ });
    emitPulse(ctx, dest, PULSE_FREQ, PULSE_DURATION_S);
    await sleep(PULSE_INTERVAL_MS);
  }

  await sleep(500);
  stopDetector();

  const correlated = correlatePulseSequence(pulseEmitTimes, rawDetections, {
    ...DEFAULT_PULSE_DETECTION_CONFIG,
    targetFrequencyHz: PULSE_FREQ,
    pulseIntervalMs: PULSE_INTERVAL_MS,
  });
  const stats = computeDetectionStats(correlated);
  const pulsesDetected = correlated.length;

  const statsB = await pcB.getStats();
  let bytesSent = 0;
  let bytesReceived = 0;
  let packetsLost = 0;
  statsB.forEach((report) => {
    if (report.type === "outbound-rtp") bytesSent += (report as { bytesSent?: number }).bytesSent ?? 0;
    if (report.type === "inbound-rtp") {
      bytesReceived += (report as { bytesReceived?: number }).bytesReceived ?? 0;
      packetsLost += (report as { packetsLost?: number }).packetsLost ?? 0;
    }
  });

  const mediaPathLive = bytesReceived > 0 && remoteStream !== null;
  const observationCompletedAt = new Date().toISOString();
  const observationWindowMs = performance.now() - observationStartMs;

  pcA.close();
  pcB.close();
  await ctx.close();

  let outcome: HarnessOutcome = "pass";
  let limitationReason: string | undefined;
  if (pulsesDetected === 0 && mediaPathLive) {
    outcome = "harness_limitation";
    limitationReason =
      "headless Chromium delivers inbound RTP but decoded remote media is not exposed to WebAudio analysers";
  } else if (pulsesDetected === 0) {
    outcome = "harness_limitation";
    limitationReason = "no decoded pulse detections; inbound media path may be inactive";
  }

  const relativeEmit = pulseEmitTimes[0]
    ? pulseEmitTimes[0].emitAtMs - observationStartMs
    : null;
  const relativeDetect = correlated[0]
    ? correlated[0].detectedAtMs - observationStartMs
    : null;

  return {
    outcome,
    limitationReason,
    observationStartedAt,
    observationCompletedAt,
    runId,
    timing: {
      pulseEmitMs: relativeEmit,
      pulseDetectMs: relativeDetect,
      loopbackLatencyMs: stats.meanLatencyMs,
      datachannelRttMs: dcRtt,
      iceGatheringMs,
      connectionSetupMs,
      observationWindowMs,
    },
    syntheticPulse: {
      pulsesEmitted: PULSE_COUNT,
      pulsesDetected,
      detectionRate: pulsesDetected / PULSE_COUNT,
      meanDetectionLatencyMs: stats.meanLatencyMs,
      jitterMs: stats.jitterMs,
      correlatedPairs: pulsesDetected,
      pulseRecords: pulseEmitTimes.map((emit) => {
        const pair = correlated.find((c) => c.index === emit.index);
        return {
          index: emit.index,
          emitAtMs: emit.emitAtMs - observationStartMs,
          detectedAtMs: pair ? pair.detectedAtMs - observationStartMs : null,
          latencyMs: pair?.latencyMs ?? null,
          frequencyHz: emit.frequencyHz,
        };
      }),
    },
    transport: { bytesSent, bytesReceived, packetsLost },
    mediaPathLive,
    decodedPulseDetectorUsed: true,
  };
}

function startDecodedMediaDetector(
  ctx: AudioContext,
  remoteStream: MediaStream,
  targetFreq: number,
  onDetection: (record: PulseDetectionRecord) => void,
): () => void {
  const source = ctx.createMediaStreamSource(remoteStream);
  const analyser = ctx.createAnalyser();
  analyser.fftSize = 2048;
  source.connect(analyser);
  const silent = ctx.createGain();
  silent.gain.value = 0;
  analyser.connect(silent);
  silent.connect(ctx.destination);

  const timeData = new Float32Array(analyser.fftSize);
  const freqData = new Float32Array(analyser.frequencyBinCount);
  let lastPeakAt = 0;
  let running = true;
  const config = { ...DEFAULT_PULSE_DETECTION_CONFIG, targetFrequencyHz: targetFreq };

  const timer = setInterval(() => {
    if (!running) return;
    analyser.getFloatTimeDomainData(timeData);
    analyser.getFloatFrequencyData(freqData);

    let peak = 0;
    for (let i = 0; i < timeData.length; i++) {
      peak = Math.max(peak, Math.abs(timeData[i]!));
    }

    const bandEnergy = bandEnergyNearFrequency(
      freqData,
      ctx.sampleRate,
      targetFreq,
      config.frequencyToleranceHz,
    );
    const frequencyHz = dominantFrequencyHz(freqData, ctx.sampleRate);
    const now = performance.now();

    if (
      peak > config.peakThreshold &&
      bandEnergy >= config.minBandEnergy &&
      now - lastPeakAt > config.peakCooldownMs
    ) {
      lastPeakAt = now;
      onDetection({ detectedAtMs: now, bandEnergy, peakAmplitude: peak, frequencyHz });
    }
  }, 5);

  return () => {
    running = false;
    clearInterval(timer);
  };
}

function waitForDataChannelRtt(
  dcA: RTCDataChannel,
  pcB: RTCPeerConnection,
  timeoutMs: number,
): Promise<number | null> {
  return new Promise((resolve) => {
    let settled = false;
    const finish = (value: number | null) => {
      if (settled) return;
      settled = true;
      resolve(value);
    };

    const timer = setTimeout(() => finish(null), timeoutMs);

    dcA.onopen = () => {
      const sentAt = performance.now();
      dcA.send(JSON.stringify({ type: "ping", sentAt }));
    };

    pcB.ondatachannel = (e) => {
      e.channel.onmessage = (msg) => {
        try {
          const data = JSON.parse(msg.data as string) as { type: string; sentAt: number };
          if (data.type === "ping") {
            e.channel.send(JSON.stringify({ type: "pong", sentAt: data.sentAt }));
          }
        } catch {
          /* ignore */
        }
      };
    };

    dcA.onmessage = (msg) => {
      try {
        const data = JSON.parse(msg.data as string) as { type: string; sentAt: number };
        if (data.type === "pong") {
          clearTimeout(timer);
          finish(performance.now() - data.sentAt);
        }
      } catch {
        /* ignore */
      }
    };
  });
}

function buildFailureResult(
  runId: string,
  observationStartedAt: string,
  observationCompletedAt: string,
  observationStartMs: number,
  outcome: HarnessOutcome,
  limitationReason: string,
): SyntheticRunResult {
  void observationStartMs;
  return {
    outcome,
    limitationReason,
    observationStartedAt,
    observationCompletedAt,
    runId,
    timing: {
      pulseEmitMs: null,
      pulseDetectMs: null,
      loopbackLatencyMs: null,
      datachannelRttMs: null,
      iceGatheringMs: null,
      connectionSetupMs: null,
      observationWindowMs: null,
    },
    syntheticPulse: {
      pulsesEmitted: 0,
      pulsesDetected: 0,
      detectionRate: 0,
      meanDetectionLatencyMs: null,
      jitterMs: null,
      correlatedPairs: 0,
      pulseRecords: [],
    },
    transport: { bytesSent: 0, bytesReceived: 0, packetsLost: 0 },
    mediaPathLive: false,
    decodedPulseDetectorUsed: true,
  };
}

function receiverAudioStream(pc: RTCPeerConnection): MediaStream | null {
  const tracks = pc
    .getReceivers()
    .map((receiver) => receiver.track)
    .filter((track): track is MediaStreamTrack => track?.kind === "audio");
  return tracks.length > 0 ? new MediaStream(tracks) : null;
}

function waitForRemoteTrack(pc: RTCPeerConnection, timeoutMs: number): Promise<MediaStream | null> {
  return new Promise((resolve) => {
    const existing = receiverAudioStream(pc);
    if (existing) {
      resolve(existing);
      return;
    }
    const timer = setTimeout(() => resolve(receiverAudioStream(pc)), timeoutMs);
    pc.ontrack = (e) => {
      clearTimeout(timer);
      resolve(e.streams[0] ?? receiverAudioStream(pc));
    };
  });
}

async function waitForInboundBytes(pc: RTCPeerConnection, timeoutMs: number): Promise<void> {
  const deadline = performance.now() + timeoutMs;
  while (performance.now() < deadline) {
    if ((await inboundBytes(pc)) > 0) return;
    await sleep(100);
  }
}

async function inboundBytes(pc: RTCPeerConnection): Promise<number> {
  const stats = await pc.getStats();
  let received = 0;
  stats.forEach((report) => {
    if (report.type === "inbound-rtp") {
      received += (report as { bytesReceived?: number }).bytesReceived ?? 0;
    }
  });
  return received;
}

function emitPulse(
  ctx: AudioContext,
  dest: MediaStreamAudioDestinationNode,
  freq: number,
  duration: number,
): void {
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.frequency.value = freq;
  gain.gain.value = 1.0;
  osc.connect(gain);
  gain.connect(dest);
  const now = ctx.currentTime;
  osc.start(now);
  osc.stop(now + duration);
}

function waitForConnection(pc: RTCPeerConnection, timeoutMs: number): Promise<boolean> {
  return new Promise((resolve) => {
    if (pc.connectionState === "connected") {
      resolve(true);
      return;
    }
    const timer = setTimeout(() => resolve(pc.connectionState === "connected"), timeoutMs);
    const check = () => {
      if (pc.connectionState === "connected") {
        clearTimeout(timer);
        pc.removeEventListener("connectionstatechange", check);
        resolve(true);
      } else if (pc.connectionState === "failed") {
        clearTimeout(timer);
        pc.removeEventListener("connectionstatechange", check);
        resolve(false);
      }
    };
    pc.addEventListener("connectionstatechange", check);
  });
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

export function buildPerformanceRunDraft(result: SyntheticRunResult): PerformanceRunDraft {
  const pathOk =
    result.outcome === "pass" &&
    result.mediaPathLive &&
    result.syntheticPulse.pulsesDetected >= 4;

  const harnessLimitation =
    result.outcome === "harness_limitation"
      ? result.limitationReason ?? "decoded media detection unavailable"
      : null;

  return {
    schemaVersion: "PerformanceRunV1",
    evidenceLevel: "browser_synthetic_media_path",
    evidenceStatus: "exploratory_non_authoritative",
    metadata: {
      runId: result.runId,
      startedAt: result.observationStartedAt,
      completedAt: result.observationCompletedAt,
      harnessVersion: "0.3.0",
      browserFamily: detectBrowserFamily(),
      platform: navigator.platform ?? null,
      ...(harnessLimitation ? { harnessLimitation } : {}),
    },
    timing: {
      pulseEmitMs: metricOrUnavailable(result.timing.pulseEmitMs, "no pulse emit timing"),
      pulseDetectMs: pathOk
        ? metricOrUnavailable(result.timing.pulseDetectMs, "no pulse detection timing")
        : unavailable(harnessLimitation ?? "inbound media path did not deliver detectable pulses"),
      loopbackLatencyMs: pathOk
        ? metricOrUnavailable(result.timing.loopbackLatencyMs, "no correlated loopback latency")
        : unavailable(harnessLimitation ?? "no pulse detections for loopback latency"),
      datachannelRttMs: result.timing.datachannelRttMs !== null
        ? observed(result.timing.datachannelRttMs)
        : unavailable("ping/pong RTT not completed"),
      iceGatheringMs: result.timing.iceGatheringMs !== null
        ? observed(result.timing.iceGatheringMs)
        : unavailable("ICE gathering timing unavailable"),
      connectionSetupMs: result.timing.connectionSetupMs !== null
        ? observed(result.timing.connectionSetupMs)
        : unavailable("connection setup timing unavailable"),
      observationWindowMs: result.timing.observationWindowMs !== null
        ? observed(result.timing.observationWindowMs)
        : unavailable("observation window unavailable"),
    },
    syntheticPulse: {
      pulsesEmitted: observed(result.syntheticPulse.pulsesEmitted),
      pulsesDetected: observed(result.syntheticPulse.pulsesDetected),
      detectionRate: observed(result.syntheticPulse.detectionRate),
      meanDetectionLatencyMs: pathOk
        ? metricOrUnavailable(result.syntheticPulse.meanDetectionLatencyMs, "no detection latency samples")
        : unavailable(harnessLimitation ?? "no pulse detections"),
      jitterMs: pathOk
        ? metricOrUnavailable(result.syntheticPulse.jitterMs, "insufficient samples for jitter")
        : unavailable(harnessLimitation ?? "no pulse detections"),
    },
    transport: {
      candidatePairType: unsupported(),
      bytesSent: observed(result.transport.bytesSent),
      bytesReceived: observed(result.transport.bytesReceived),
      packetsLost: observed(result.transport.packetsLost),
    },
  };
}

function metricOrUnavailable(value: number | null, reason: string) {
  return value === null ? unavailable(reason) : observed(value);
}

function detectBrowserFamily(): string | null {
  const ua = navigator.userAgent;
  if (ua.includes("Chrome")) return "chromium";
  if (ua.includes("Firefox")) return "firefox";
  if (ua.includes("Safari")) return "safari";
  return null;
}

export {
  correlatePulseSequence,
  classifyDetection,
  rejectBackgroundRtpWithoutPulse,
  bandEnergyNearFrequency,
  DEFAULT_PULSE_DETECTION_CONFIG,
} from "./pulse-detector";
