import {
  observed,
  unavailable,
  unsupported,
  type PerformanceRunDraft,
} from "../../../shared/performance/types";

export interface SyntheticRunResult {
  timing: {
    pulseEmitMs: number | null;
    pulseDetectMs: number | null;
    loopbackLatencyMs: number | null;
    datachannelRttMs: number;
    iceGatheringMs: number;
    connectionSetupMs: number;
  };
  syntheticPulse: {
    pulsesEmitted: number;
    pulsesDetected: number;
    detectionRate: number;
    meanDetectionLatencyMs: number | null;
    jitterMs: number | null;
    correlatedPairs: number;
  };
  transport: {
    bytesSent: number;
    bytesReceived: number;
    packetsLost: number;
  };
  mediaPathLive: boolean;
}

const PULSE_COUNT = 5;
const PULSE_INTERVAL_MS = 200;
const PULSE_FREQ = 1000;
const PULSE_DURATION_S = 0.1;
const INBOUND_WAIT_MS = 5000;
const ONTACK_WAIT_MS = 5000;
const PEAK_THRESHOLD = 0.015;
const PEAK_COOLDOWN_MS = 80;

export async function runSyntheticLoopback(): Promise<SyntheticRunResult> {
  const setupStart = performance.now();

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

  const iceStart = performance.now();
  const stream = dest.stream;
  for (const track of stream.getTracks()) {
    pcA.addTrack(track, stream);
  }

  const remoteTrackReady = waitForRemoteTrack(pcB, ONTACK_WAIT_MS);

  const dcA = pcA.createDataChannel("probe");
  let dcRtt = 0;
  const dcReady = new Promise<void>((resolve) => {
    dcA.onopen = () => {
      const sent = performance.now();
      dcA.send(JSON.stringify({ type: "ping", sentAt: sent }));
    };
    pcB.ondatachannel = (e) => {
      e.channel.onmessage = (msg) => {
        try {
          const data = JSON.parse(msg.data as string) as { type: string; sentAt: number };
          if (data.type === "ping") {
            dcRtt = performance.now() - data.sentAt;
            e.channel.send(JSON.stringify({ type: "pong", sentAt: data.sentAt }));
          }
        } catch {
          /* ignore */
        }
      };
    };
    setTimeout(resolve, 100);
  });

  const offer = await pcA.createOffer();
  await pcA.setLocalDescription(offer);
  await pcB.setRemoteDescription(offer);
  const answer = await pcB.createAnswer();
  await pcB.setLocalDescription(answer);
  await pcA.setRemoteDescription(answer);

  await waitForConnection(pcA);
  const iceGatheringMs = performance.now() - iceStart;
  const connectionSetupMs = performance.now() - setupStart;
  await dcReady;

  let remoteStream = await remoteTrackReady;
  remoteStream ??= receiverAudioStream(pcB);
  await waitForInboundBytes(pcB, INBOUND_WAIT_MS);

  const detections: { pulseIndex: number; detectedAtMs: number }[] = [];
  let stopDetector = () => {};
  if (remoteStream) {
    stopDetector = startPeakDetector(ctx, remoteStream, (detectedAtMs) => {
      detections.push({ pulseIndex: detections.length, detectedAtMs });
    });
    await sleep(300);
  }

  const pulseEmitTimes: { index: number; emitAtMs: number }[] = [];
  for (let i = 0; i < PULSE_COUNT; i++) {
    const beforeBytes = await inboundBytes(pcB);
    const emitAtMs = performance.now();
    pulseEmitTimes.push({ index: i, emitAtMs });
    emitPulse(ctx, dest, PULSE_FREQ, PULSE_DURATION_S);
    const byteDetection = await waitForInboundByteIncrease(pcB, beforeBytes, 400);
    if (byteDetection !== null && detections.every((d) => Math.abs(d.detectedAtMs - byteDetection) > 5)) {
      detections.push({ pulseIndex: detections.length, detectedAtMs: byteDetection });
    }
    await sleep(PULSE_INTERVAL_MS);
  }

  await sleep(500);
  stopDetector();

  const correlated = correlatePulses(pulseEmitTimes, detections);
  const pulsesDetected = correlated.length;
  const latencies = correlated.map((c) => c.detectedAtMs - c.emitAtMs);
  const meanLatency =
    latencies.length > 0 ? latencies.reduce((a, b) => a + b, 0) / latencies.length : null;
  const jitter =
    latencies.length > 1 && meanLatency !== null
      ? Math.sqrt(latencies.reduce((s, l) => s + (l - meanLatency) ** 2, 0) / latencies.length)
      : null;

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

  pcA.close();
  pcB.close();
  await ctx.close();

  return {
    timing: {
      pulseEmitMs: pulseEmitTimes[0]?.emitAtMs ?? null,
      pulseDetectMs: correlated[0]?.detectedAtMs ?? null,
      loopbackLatencyMs: meanLatency,
      datachannelRttMs: dcRtt,
      iceGatheringMs,
      connectionSetupMs,
    },
    syntheticPulse: {
      pulsesEmitted: PULSE_COUNT,
      pulsesDetected,
      detectionRate: pulsesDetected / PULSE_COUNT,
      meanDetectionLatencyMs: meanLatency,
      jitterMs: jitter,
      correlatedPairs: pulsesDetected,
    },
    transport: { bytesSent, bytesReceived, packetsLost },
    mediaPathLive,
  };
}

function startPeakDetector(
  ctx: AudioContext,
  remoteStream: MediaStream,
  onPeak: (detectedAtMs: number) => void,
): () => void {
  const source = ctx.createMediaStreamSource(remoteStream);
  const analyser = ctx.createAnalyser();
  analyser.fftSize = 2048;
  source.connect(analyser);
  const silent = ctx.createGain();
  silent.gain.value = 0;
  analyser.connect(silent);
  silent.connect(ctx.destination);

  const data = new Float32Array(analyser.fftSize);
  let lastPeakAt = 0;
  let running = true;

  const timer = setInterval(() => {
    if (!running) return;
    analyser.getFloatTimeDomainData(data);
    let peak = 0;
    for (let i = 0; i < data.length; i++) {
      peak = Math.max(peak, Math.abs(data[i]!));
    }
    const now = performance.now();
    if (peak > PEAK_THRESHOLD && now - lastPeakAt > PEAK_COOLDOWN_MS) {
      lastPeakAt = now;
      onPeak(now);
    }
  }, 5);

  return () => {
    running = false;
    clearInterval(timer);
  };
}

function receiverAudioStream(pc: RTCPeerConnection): MediaStream | null {
  const tracks = pc
    .getReceivers()
    .map((receiver) => receiver.track)
    .filter((track): track is MediaStreamTrack => track?.kind === "audio");
  return tracks.length > 0 ? new MediaStream(tracks) : null;
}

function correlatePulses(
  emits: { index: number; emitAtMs: number }[],
  detections: { pulseIndex: number; detectedAtMs: number }[],
): { index: number; emitAtMs: number; detectedAtMs: number }[] {
  const sorted = [...detections].sort((a, b) => a.detectedAtMs - b.detectedAtMs);
  const pairs: { index: number; emitAtMs: number; detectedAtMs: number }[] = [];
  let detectionCursor = 0;

  for (const emit of emits) {
    while (
      detectionCursor < sorted.length &&
      sorted[detectionCursor]!.detectedAtMs < emit.emitAtMs
    ) {
      detectionCursor += 1;
    }
    const candidate = sorted[detectionCursor];
    if (!candidate) break;
    const latency = candidate.detectedAtMs - emit.emitAtMs;
    if (latency >= 0 && latency <= PULSE_INTERVAL_MS * 2) {
      pairs.push({
        index: emit.index,
        emitAtMs: emit.emitAtMs,
        detectedAtMs: candidate.detectedAtMs,
      });
      detectionCursor += 1;
    }
  }

  return pairs;
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

async function waitForInboundByteIncrease(
  pc: RTCPeerConnection,
  beforeBytes: number,
  timeoutMs: number,
): Promise<number | null> {
  const deadline = performance.now() + timeoutMs;
  while (performance.now() < deadline) {
    const after = await inboundBytes(pc);
    if (after > beforeBytes) return performance.now();
    await sleep(5);
  }
  return null;
}

function emitPulse(ctx: AudioContext, dest: MediaStreamAudioDestinationNode, freq: number, duration: number): void {
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

function waitForConnection(pc: RTCPeerConnection): Promise<void> {
  return new Promise((resolve) => {
    if (pc.connectionState === "connected") {
      resolve();
      return;
    }
    const check = () => {
      if (pc.connectionState === "connected" || pc.connectionState === "failed") {
        pc.removeEventListener("connectionstatechange", check);
        resolve();
      }
    };
    pc.addEventListener("connectionstatechange", check);
    setTimeout(resolve, 3000);
  });
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

export function buildPerformanceRunDraft(result: SyntheticRunResult): PerformanceRunDraft {
  const pathOk = result.mediaPathLive && result.syntheticPulse.pulsesDetected > 0;

  return {
    schemaVersion: "PerformanceRunV1",
    evidenceLevel: "browser_synthetic_media_path",
    evidenceStatus: "exploratory_non_authoritative",
    metadata: {
      runId: `synthetic-${Date.now()}`,
      startedAt: new Date().toISOString(),
      completedAt: new Date().toISOString(),
      harnessVersion: "0.2.0",
      browserFamily: detectBrowserFamily(),
      platform: navigator.platform ?? null,
    },
    timing: {
      pulseEmitMs: metricOrUnavailable(result.timing.pulseEmitMs, "no pulse emit timing"),
      pulseDetectMs: pathOk
        ? metricOrUnavailable(result.timing.pulseDetectMs, "no pulse detection timing")
        : unavailable("inbound media path did not deliver detectable pulses"),
      loopbackLatencyMs: pathOk
        ? metricOrUnavailable(result.timing.loopbackLatencyMs, "no correlated loopback latency")
        : unavailable("no pulse detections for loopback latency"),
      datachannelRttMs: observed(result.timing.datachannelRttMs),
      iceGatheringMs: observed(result.timing.iceGatheringMs),
      connectionSetupMs: observed(result.timing.connectionSetupMs),
    },
    syntheticPulse: {
      pulsesEmitted: observed(result.syntheticPulse.pulsesEmitted),
      pulsesDetected: observed(result.syntheticPulse.pulsesDetected),
      detectionRate: observed(result.syntheticPulse.detectionRate),
      meanDetectionLatencyMs: pathOk
        ? metricOrUnavailable(result.syntheticPulse.meanDetectionLatencyMs, "no detection latency samples")
        : unavailable("no pulse detections"),
      jitterMs: pathOk
        ? metricOrUnavailable(result.syntheticPulse.jitterMs, "insufficient samples for jitter")
        : unavailable("no pulse detections"),
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
