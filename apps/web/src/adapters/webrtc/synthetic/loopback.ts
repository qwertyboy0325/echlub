import {
  observed,
  unsupported,
  type PerformanceRunDraft,
} from "../../../shared/performance/types";

export interface SyntheticRunResult {
  timing: {
    pulseEmitMs: number;
    pulseDetectMs: number;
    loopbackLatencyMs: number;
    datachannelRttMs: number;
    iceGatheringMs: number;
    connectionSetupMs: number;
  };
  syntheticPulse: {
    pulsesEmitted: number;
    pulsesDetected: number;
    detectionRate: number;
    meanDetectionLatencyMs: number;
    jitterMs: number;
  };
  transport: {
    bytesSent: number;
    bytesReceived: number;
    packetsLost: number;
  };
}

const PULSE_COUNT = 5;
const PULSE_INTERVAL_MS = 200;
const PULSE_FREQ = 1000;
const PULSE_DURATION_S = 0.05;

export async function runSyntheticLoopback(): Promise<SyntheticRunResult> {
  const setupStart = performance.now();

  const ctx = new AudioContext();
  await ctx.audioWorklet.addModule("/audio-worklets/pulse-detector.js");

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

  let remoteStream: MediaStream | null = null;
  pcB.ontrack = (e) => {
    remoteStream = e.streams[0];
  };

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

  const detectorNode = new AudioWorkletNode(ctx, "pulse-detector");
  const detections: number[] = [];
  detectorNode.port.onmessage = (e) => {
    if (e.data.type === "pulse_detected") {
      detections.push(e.data.timestamp);
    }
  };

  if (remoteStream) {
    const source = ctx.createMediaStreamSource(remoteStream);
    source.connect(detectorNode);
  }

  const pulseEmitTimes: number[] = [];
  for (let i = 0; i < PULSE_COUNT; i++) {
    const emitTime = performance.now();
    pulseEmitTimes.push(emitTime);
    emitPulse(ctx, dest, PULSE_FREQ, PULSE_DURATION_S);
    await sleep(PULSE_INTERVAL_MS);
  }

  await sleep(500);

  const pulsesDetected = detections.length;
  const latencies = detections.slice(0, pulseEmitTimes.length).map((d, i) => d - pulseEmitTimes[i]);
  const meanLatency = latencies.length > 0 ? latencies.reduce((a, b) => a + b, 0) / latencies.length : 0;
  const jitter =
    latencies.length > 1
      ? Math.sqrt(latencies.reduce((s, l) => s + (l - meanLatency) ** 2, 0) / latencies.length)
      : 0;

  const statsA = await pcA.getStats();
  let bytesSent = 0;
  let bytesReceived = 0;
  let packetsLost = 0;
  statsA.forEach((report) => {
    if (report.type === "outbound-rtp") bytesSent += (report as { bytesSent?: number }).bytesSent ?? 0;
    if (report.type === "inbound-rtp") {
      bytesReceived += (report as { bytesReceived?: number }).bytesReceived ?? 0;
      packetsLost += (report as { packetsLost?: number }).packetsLost ?? 0;
    }
  });

  pcA.close();
  pcB.close();
  await ctx.close();

  return {
    timing: {
      pulseEmitMs: pulseEmitTimes[0] ?? 0,
      pulseDetectMs: detections[0] ?? 0,
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
    },
    transport: { bytesSent, bytesReceived, packetsLost },
  };
}

function emitPulse(ctx: AudioContext, dest: MediaStreamAudioDestinationNode, freq: number, duration: number): void {
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.frequency.value = freq;
  gain.gain.value = 0.5;
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
  return {
    schemaVersion: "PerformanceRunV1",
    evidenceLevel: "browser_synthetic_media_path",
    evidenceStatus: "exploratory_non_authoritative",
    metadata: {
      runId: `synthetic-${Date.now()}`,
      startedAt: new Date().toISOString(),
      completedAt: new Date().toISOString(),
      harnessVersion: "0.1.0",
      browserFamily: detectBrowserFamily(),
      platform: navigator.platform ?? null,
    },
    timing: {
      pulseEmitMs: observed(result.timing.pulseEmitMs),
      pulseDetectMs: observed(result.timing.pulseDetectMs),
      loopbackLatencyMs: observed(result.timing.loopbackLatencyMs),
      datachannelRttMs: observed(result.timing.datachannelRttMs),
      iceGatheringMs: observed(result.timing.iceGatheringMs),
      connectionSetupMs: observed(result.timing.connectionSetupMs),
    },
    syntheticPulse: {
      pulsesEmitted: observed(result.syntheticPulse.pulsesEmitted),
      pulsesDetected: observed(result.syntheticPulse.pulsesDetected),
      detectionRate: observed(result.syntheticPulse.detectionRate),
      meanDetectionLatencyMs: observed(result.syntheticPulse.meanDetectionLatencyMs),
      jitterMs: observed(result.syntheticPulse.jitterMs),
    },
    transport: {
      candidatePairType: unsupported(),
      bytesSent: observed(result.transport.bytesSent),
      bytesReceived: observed(result.transport.bytesReceived),
      packetsLost: observed(result.transport.packetsLost),
    },
  };
}

function detectBrowserFamily(): string | null {
  const ua = navigator.userAgent;
  if (ua.includes("Chrome")) return "chromium";
  if (ua.includes("Firefox")) return "firefox";
  if (ua.includes("Safari")) return "safari";
  return null;
}
