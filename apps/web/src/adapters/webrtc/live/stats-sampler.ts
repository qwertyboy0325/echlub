import { observed, unavailable, unsupported, type MetricValue } from "./types";

export interface NormalizedStatsSample {
  offsetMs: number;
  candidatePair: Record<string, MetricValue>;
  inboundAudio: Record<string, MetricValue>;
  outboundAudio: Record<string, MetricValue>;
  remoteInboundAudio: Record<string, MetricValue>;
  codec: Record<string, MetricValue>;
  intervalMetrics?: Record<string, MetricValue>;
}

function metricFromReport(report: Record<string, unknown>, key: string): MetricValue {
  const val = report[key];
  if (typeof val === "number" && Number.isFinite(val)) {
    return observed(val);
  }
  if (val === undefined) return unsupported();
  return unavailable("not exposed");
}

function sanitizeCandidateType(report: Record<string, unknown>): MetricValue {
  const t = report.candidateType;
  if (typeof t === "string") return observed(1);
  return unsupported();
}

export async function collectNormalizedStats(
  pc: RTCPeerConnection,
  offsetMs: number,
  prev: NormalizedStatsSample | null,
): Promise<NormalizedStatsSample> {
  const stats = await pc.getStats();
  let candidatePair: Record<string, MetricValue> = {};
  let inboundAudio: Record<string, MetricValue> = {};
  let outboundAudio: Record<string, MetricValue> = {};
  let remoteInboundAudio: Record<string, MetricValue> = {};
  let codec: Record<string, MetricValue> = {};

  const reports = new Map<string, Record<string, unknown>>();
  stats.forEach((report) => {
    reports.set(report.id, report as unknown as Record<string, unknown>);
  });

  for (const report of reports.values()) {
    if (report.type === "transport") {
      const pairId = report.selectedCandidatePairId as string | undefined;
      if (pairId && reports.has(pairId)) {
        const pair = reports.get(pairId)!;
        candidatePair = {
          state: metricFromReport(pair, "state"),
          nominated: metricFromReport(pair, "nominated"),
          protocol: unavailable("category only"),
          localCandidateType: sanitizeCandidateType(
            reports.get(String(pair.localCandidateId)) ?? {},
          ),
          remoteCandidateType: sanitizeCandidateType(
            reports.get(String(pair.remoteCandidateId)) ?? {},
          ),
          currentRoundTripTime: metricFromReport(pair, "currentRoundTripTime"),
          packetsSent: metricFromReport(pair, "packetsSent"),
          packetsReceived: metricFromReport(pair, "packetsReceived"),
          bytesSent: metricFromReport(pair, "bytesSent"),
          bytesReceived: metricFromReport(pair, "bytesReceived"),
        };
      }
    }
    if (report.type === "inbound-rtp" && report.kind === "audio") {
      inboundAudio = {
        packetsReceived: metricFromReport(report, "packetsReceived"),
        packetsLost: metricFromReport(report, "packetsLost"),
        jitter: metricFromReport(report, "jitter"),
        jitterBufferDelay: metricFromReport(report, "jitterBufferDelay"),
        jitterBufferTargetDelay: metricFromReport(report, "jitterBufferTargetDelay"),
        jitterBufferMinimumDelay: metricFromReport(report, "jitterBufferMinimumDelay"),
        bytesReceived: metricFromReport(report, "bytesReceived"),
        concealedSamples: metricFromReport(report, "concealedSamples"),
        totalSamplesReceived: metricFromReport(report, "totalSamplesReceived"),
        audioLevel: metricFromReport(report, "audioLevel"),
      };
    }
    if (report.type === "outbound-rtp" && report.kind === "audio") {
      outboundAudio = {
        packetsSent: metricFromReport(report, "packetsSent"),
        bytesSent: metricFromReport(report, "bytesSent"),
        retransmittedPacketsSent: metricFromReport(report, "retransmittedPacketsSent"),
        audioLevel: metricFromReport(report, "audioLevel"),
      };
    }
    if (report.type === "remote-inbound-rtp" && report.kind === "audio") {
      remoteInboundAudio = {
        roundTripTime: metricFromReport(report, "roundTripTime"),
        fractionLost: metricFromReport(report, "fractionLost"),
        packetsLost: metricFromReport(report, "packetsLost"),
        jitter: metricFromReport(report, "jitter"),
      };
    }
    if (report.type === "codec") {
      codec = {
        mimeType: metricFromReport(report, "mimeType"),
        clockRate: metricFromReport(report, "clockRate"),
        channels: metricFromReport(report, "channels"),
      };
    }
  }

  const sample: NormalizedStatsSample = {
    offsetMs,
    candidatePair,
    inboundAudio,
    outboundAudio,
    remoteInboundAudio,
    codec,
  };

  if (prev) {
    sample.intervalMetrics = computeIntervalMetrics(prev, sample);
  }

  return sample;
}

function computeIntervalMetrics(
  prev: NormalizedStatsSample,
  curr: NormalizedStatsSample,
): Record<string, MetricValue> {
  const dt = curr.offsetMs - prev.offsetMs;
  if (dt <= 0) return {};
  const metrics: Record<string, MetricValue> = {};
  const rtt = curr.candidatePair.currentRoundTripTime;
  if (rtt.kind === "observed" && rtt.value !== undefined) {
    metrics.candidatePairRttMs = observed(rtt.value * 1000);
  }
  const recvDelta = deltaMetric(prev.inboundAudio.bytesReceived, curr.inboundAudio.bytesReceived);
  if (recvDelta !== null) {
    metrics.receiveBitrateBps = observed((recvDelta * 8 * 1000) / dt);
  }
  const sendDelta = deltaMetric(prev.outboundAudio.bytesSent, curr.outboundAudio.bytesSent);
  if (sendDelta !== null) {
    metrics.sendBitrateBps = observed((sendDelta * 8 * 1000) / dt);
  }
  const lossDelta = deltaMetric(prev.inboundAudio.packetsLost, curr.inboundAudio.packetsLost);
  if (lossDelta !== null) {
    metrics.packetLossDelta = observed(lossDelta);
  }
  return metrics;
}

function deltaMetric(prev: MetricValue, curr: MetricValue): number | null {
  if (prev.kind !== "observed" || curr.kind !== "observed") return null;
  const delta = (curr.value ?? 0) - (prev.value ?? 0);
  return delta >= 0 ? delta : null;
}

export class StatsSampler {
  private timer: ReturnType<typeof setInterval> | null = null;
  private samples: NormalizedStatsSample[] = [];
  private prev: NormalizedStatsSample | null = null;
  private startMs = 0;

  constructor(
    private readonly pc: RTCPeerConnection,
    private readonly onSample: (sample: NormalizedStatsSample) => void,
  ) {}

  start(intervalMs: number, maxSamples: number, durationMs: number): void {
    this.stop();
    this.samples = [];
    this.prev = null;
    this.startMs = performance.now();
    const tick = async () => {
      if (this.samples.length >= maxSamples) {
        this.stop();
        return;
      }
      const offsetMs = performance.now() - this.startMs;
      if (offsetMs > durationMs) {
        this.stop();
        return;
      }
      const sample = await collectNormalizedStats(this.pc, offsetMs, this.prev);
      this.prev = sample;
      this.samples.push(sample);
      this.onSample(sample);
    };
    void tick();
    this.timer = setInterval(() => void tick(), intervalMs);
  }

  stop(): void {
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  getSamples(): NormalizedStatsSample[] {
    return this.samples;
  }
}
