import {
  invalid,
  isObservedNumber,
  observedBoolean,
  observedCategory,
  observedNumber,
  unsupported,
  unavailable,
  type MetricValue,
} from "./types";

export interface NormalizedStatsSample {
  offsetMs: number;
  candidatePair: Record<string, MetricValue>;
  inboundAudio: Record<string, MetricValue>;
  outboundAudio: Record<string, MetricValue>;
  remoteInboundAudio: Record<string, MetricValue>;
  codec: Record<string, MetricValue>;
  intervalMetrics?: Record<string, MetricValue>;
}

const CANDIDATE_TYPES = new Set(["host", "srflx", "prflx", "relay"]);
const CONNECTION_STATES = new Set(["succeeded", "in-progress"]);
const TRANSPORT_PROTOCOLS = new Set(["udp", "tcp"]);

function metricNumberFromReport(report: Record<string, unknown>, key: string): MetricValue {
  const val = report[key];
  if (typeof val === "number" && Number.isFinite(val)) {
    return observedNumber(val);
  }
  if (val === undefined) return unsupported();
  return invalid("expected finite number");
}

function metricBooleanFromReport(report: Record<string, unknown>, key: string): MetricValue {
  const val = report[key];
  if (typeof val === "boolean") {
    return observedBoolean(val);
  }
  if (val === undefined) return unsupported();
  return invalid("expected boolean");
}

function sanitizeCandidateType(report: Record<string, unknown>): MetricValue {
  const t = report.candidateType;
  if (typeof t === "string" && CANDIDATE_TYPES.has(t)) {
    return observedCategory(t);
  }
  return unsupported();
}

function sanitizeConnectionState(report: Record<string, unknown>): MetricValue {
  const state = report.state;
  if (typeof state === "string" && CONNECTION_STATES.has(state)) {
    return observedCategory(state);
  }
  if (typeof state === "string") {
    return invalid(`unexpected connection state: ${state}`);
  }
  return unsupported();
}

function sanitizeProtocol(report: Record<string, unknown>): MetricValue {
  const protocol = report.protocol;
  if (typeof protocol === "string" && TRANSPORT_PROTOCOLS.has(protocol)) {
    return observedCategory(protocol);
  }
  return unsupported();
}

function sanitizeCodecMime(report: Record<string, unknown>): MetricValue {
  const mime = report.mimeType;
  if (typeof mime === "string" && mime.startsWith("audio/")) {
    return observedCategory(mime);
  }
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
          state: sanitizeConnectionState(pair),
          nominated: metricBooleanFromReport(pair, "nominated"),
          protocol: sanitizeProtocol(pair),
          localCandidateType: sanitizeCandidateType(
            reports.get(String(pair.localCandidateId)) ?? {},
          ),
          remoteCandidateType: sanitizeCandidateType(
            reports.get(String(pair.remoteCandidateId)) ?? {},
          ),
          currentRoundTripTime: metricNumberFromReport(pair, "currentRoundTripTime"),
          packetsSent: metricNumberFromReport(pair, "packetsSent"),
          packetsReceived: metricNumberFromReport(pair, "packetsReceived"),
          bytesSent: metricNumberFromReport(pair, "bytesSent"),
          bytesReceived: metricNumberFromReport(pair, "bytesReceived"),
        };
      }
    }
    if (report.type === "inbound-rtp" && report.kind === "audio") {
      inboundAudio = {
        packetsReceived: metricNumberFromReport(report, "packetsReceived"),
        packetsLost: metricNumberFromReport(report, "packetsLost"),
        jitter: metricNumberFromReport(report, "jitter"),
        jitterBufferDelay: metricNumberFromReport(report, "jitterBufferDelay"),
        jitterBufferTargetDelay: metricNumberFromReport(report, "jitterBufferTargetDelay"),
        jitterBufferMinimumDelay: metricNumberFromReport(report, "jitterBufferMinimumDelay"),
        bytesReceived: metricNumberFromReport(report, "bytesReceived"),
        concealedSamples: metricNumberFromReport(report, "concealedSamples"),
        totalSamplesReceived: metricNumberFromReport(report, "totalSamplesReceived"),
        audioLevel: metricNumberFromReport(report, "audioLevel"),
      };
    }
    if (report.type === "outbound-rtp" && report.kind === "audio") {
      outboundAudio = {
        packetsSent: metricNumberFromReport(report, "packetsSent"),
        bytesSent: metricNumberFromReport(report, "bytesSent"),
        retransmittedPacketsSent: metricNumberFromReport(report, "retransmittedPacketsSent"),
        audioLevel: metricNumberFromReport(report, "audioLevel"),
      };
    }
    if (report.type === "remote-inbound-rtp" && report.kind === "audio") {
      remoteInboundAudio = {
        roundTripTime: metricNumberFromReport(report, "roundTripTime"),
        fractionLost: metricNumberFromReport(report, "fractionLost"),
        packetsLost: metricNumberFromReport(report, "packetsLost"),
        jitter: metricNumberFromReport(report, "jitter"),
      };
    }
    if (report.type === "codec") {
      codec = {
        mimeType: sanitizeCodecMime(report),
        clockRate: metricNumberFromReport(report, "clockRate"),
        channels: metricNumberFromReport(report, "channels"),
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

export async function collectStatsPreflight(
  pc: RTCPeerConnection,
): Promise<NormalizedStatsSample> {
  return collectNormalizedStats(pc, 0, null);
}

export function deltaCumulativeMetric(prev: MetricValue, curr: MetricValue): MetricValue {
  if (prev.kind === "invalid" || curr.kind === "invalid") {
    return invalid("expected observed_number");
  }
  if (prev.kind === "unsupported" || curr.kind === "unsupported") {
    return unsupported();
  }
  if (prev.kind === "unavailable") {
    return unavailable("missing previous value");
  }
  if (curr.kind === "unavailable") {
    return unavailable("missing current value");
  }
  if (!isObservedNumber(prev) || !isObservedNumber(curr)) {
    return invalid("expected observed_number");
  }
  const delta = curr.value - prev.value;
  if (delta < 0) {
    return invalid("counter_reset");
  }
  return observedNumber(delta);
}

export function computeIntervalMetrics(
  prev: NormalizedStatsSample,
  curr: NormalizedStatsSample,
): Record<string, MetricValue> {
  const dt = curr.offsetMs - prev.offsetMs;
  if (dt <= 0) return {};
  const metrics: Record<string, MetricValue> = {};
  const rtt = curr.candidatePair.currentRoundTripTime;
  if (rtt !== undefined && isObservedNumber(rtt)) {
    metrics.candidatePairRttMs = observedNumber(rtt.value * 1000);
  }
  const recvDelta = deltaCumulativeMetric(
    prev.inboundAudio.bytesReceived ?? unsupported(),
    curr.inboundAudio.bytesReceived ?? unsupported(),
  );
  if (recvDelta.kind === "observed_number") {
    metrics.receiveBitrateBps = observedNumber((recvDelta.value * 8 * 1000) / dt);
  } else {
    metrics.receiveBitrateBps = recvDelta;
  }
  const sendDelta = deltaCumulativeMetric(
    prev.outboundAudio.bytesSent ?? unsupported(),
    curr.outboundAudio.bytesSent ?? unsupported(),
  );
  if (sendDelta.kind === "observed_number") {
    metrics.sendBitrateBps = observedNumber((sendDelta.value * 8 * 1000) / dt);
  } else {
    metrics.sendBitrateBps = sendDelta;
  }
  const lossDelta = deltaCumulativeMetric(
    prev.inboundAudio.packetsLost ?? unsupported(),
    curr.inboundAudio.packetsLost ?? unsupported(),
  );
  if (lossDelta.kind === "observed_number") {
    metrics.packetLossDelta = lossDelta;
  } else {
    metrics.packetLossDelta = lossDelta;
  }
  return metrics;
}

export class StatsSampler {
  private timer: ReturnType<typeof setInterval> | null = null;
  private samples: NormalizedStatsSample[] = [];
  private prev: NormalizedStatsSample | null = null;
  private startMs = 0;
  private sampling = false;
  private generation = 0;
  private activeGeneration = 0;
  private stopped = false;

  constructor(
    private readonly pc: RTCPeerConnection,
    private readonly onSample: (sample: NormalizedStatsSample) => void,
  ) {}

  start(intervalMs: number, maxSamples: number, durationMs: number): void {
    this.stop();
    this.stopped = false;
    this.generation += 1;
    this.activeGeneration = this.generation;
    const token = this.activeGeneration;
    this.samples = [];
    this.prev = null;
    this.startMs = performance.now();

    const tick = async () => {
      if (this.stopped || token !== this.activeGeneration) return;
      if (this.sampling) return;
      this.sampling = true;
      try {
        if (token !== this.activeGeneration || this.stopped) return;
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
        if (token !== this.activeGeneration || this.stopped) return;
        this.prev = sample;
        this.samples.push(sample);
        this.onSample(sample);
      } finally {
        this.sampling = false;
      }
    };
    void tick();
    this.timer = setInterval(() => void tick(), intervalMs);
  }

  stop(): void {
    this.stopped = true;
    this.activeGeneration += 1;
    if (this.timer) {
      clearInterval(this.timer);
      this.timer = null;
    }
  }

  getSamples(): NormalizedStatsSample[] {
    return this.samples;
  }
}
