export interface PulseEmitRecord {
  index: number;
  emitAtMs: number;
  frequencyHz: number;
}

export interface PulseDetectionRecord {
  detectedAtMs: number;
  bandEnergy: number;
  peakAmplitude: number;
  frequencyHz: number;
}

export interface PulseCorrelationResult {
  index: number;
  emitAtMs: number;
  detectedAtMs: number;
  latencyMs: number;
}

export type DetectionRejectReason =
  | "accepted"
  | "reject_before_emit"
  | "reject_duplicate"
  | "reject_wrong_frequency"
  | "reject_false_positive"
  | "reject_between_pulses";

export interface PulseDetectionConfig {
  targetFrequencyHz: number;
  frequencyToleranceHz: number;
  minBandEnergy: number;
  peakThreshold: number;
  peakCooldownMs: number;
  maxCorrelationLatencyMs: number;
  pulseIntervalMs: number;
}

export const DEFAULT_PULSE_DETECTION_CONFIG: PulseDetectionConfig = {
  targetFrequencyHz: 1000,
  frequencyToleranceHz: 150,
  minBandEnergy: 0.0005,
  peakThreshold: 0.015,
  peakCooldownMs: 80,
  maxCorrelationLatencyMs: 400,
  pulseIntervalMs: 200,
};

export function bandEnergyNearFrequency(
  fftMagnitudes: Float32Array,
  sampleRate: number,
  targetHz: number,
  toleranceHz: number,
): number {
  const binWidth = sampleRate / (fftMagnitudes.length * 2);
  const lowBin = Math.max(0, Math.floor((targetHz - toleranceHz) / binWidth));
  const highBin = Math.min(fftMagnitudes.length - 1, Math.ceil((targetHz + toleranceHz) / binWidth));
  let energy = 0;
  for (let i = lowBin; i <= highBin; i++) {
    const mag = fftMagnitudes[i] ?? 0;
    energy += mag * mag;
  }
  return energy;
}

export function dominantFrequencyHz(
  fftMagnitudes: Float32Array,
  sampleRate: number,
): number {
  let maxBin = 0;
  let maxMag = 0;
  for (let i = 1; i < fftMagnitudes.length; i++) {
    const mag = fftMagnitudes[i] ?? 0;
    if (mag > maxMag) {
      maxMag = mag;
      maxBin = i;
    }
  }
  return (maxBin * sampleRate) / (fftMagnitudes.length * 2);
}

export function classifyDetection(
  detection: PulseDetectionRecord,
  emits: PulseEmitRecord[],
  priorAccepted: PulseDetectionRecord[],
  config: PulseDetectionConfig,
): DetectionRejectReason {
  const firstEmit = emits[0]?.emitAtMs ?? Infinity;
  if (detection.detectedAtMs < firstEmit) {
    return "reject_before_emit";
  }

  const freqDelta = Math.abs(detection.frequencyHz - config.targetFrequencyHz);
  if (freqDelta > config.frequencyToleranceHz) {
    return "reject_wrong_frequency";
  }

  if (detection.bandEnergy < config.minBandEnergy || detection.peakAmplitude < config.peakThreshold) {
    return "reject_false_positive";
  }

  for (const prior of priorAccepted) {
    if (Math.abs(detection.detectedAtMs - prior.detectedAtMs) < config.peakCooldownMs) {
      return "reject_duplicate";
    }
  }

  const matchedEmit = emits.find(
    (e) =>
      detection.detectedAtMs >= e.emitAtMs &&
      detection.detectedAtMs - e.emitAtMs <= config.maxCorrelationLatencyMs,
  );
  if (!matchedEmit) {
    return "reject_between_pulses";
  }

  return "accepted";
}

export function correlatePulseSequence(
  emits: PulseEmitRecord[],
  detections: PulseDetectionRecord[],
  config: PulseDetectionConfig = DEFAULT_PULSE_DETECTION_CONFIG,
): PulseCorrelationResult[] {
  const sorted = [...detections].sort((a, b) => a.detectedAtMs - b.detectedAtMs);
  const accepted: PulseDetectionRecord[] = [];
  const pairs: PulseCorrelationResult[] = [];

  for (const detection of sorted) {
    const reason = classifyDetection(detection, emits, accepted, config);
    if (reason !== "accepted") continue;

    const emit = emits.find(
      (e) =>
        !pairs.some((p) => p.index === e.index) &&
        detection.detectedAtMs >= e.emitAtMs &&
        detection.detectedAtMs - e.emitAtMs <= config.maxCorrelationLatencyMs,
    );
    if (!emit) continue;

    accepted.push(detection);
    pairs.push({
      index: emit.index,
      emitAtMs: emit.emitAtMs,
      detectedAtMs: detection.detectedAtMs,
      latencyMs: detection.detectedAtMs - emit.emitAtMs,
    });
  }

  return pairs;
}

export function rejectBackgroundRtpWithoutPulse(
  detections: PulseDetectionRecord[],
  emits: PulseEmitRecord[],
  config: PulseDetectionConfig = DEFAULT_PULSE_DETECTION_CONFIG,
): PulseDetectionRecord[] {
  return detections.filter(
    (d) => classifyDetection(d, emits, [], config) === "accepted",
  );
}

export function computeDetectionStats(pairs: PulseCorrelationResult[]): {
  meanLatencyMs: number | null;
  jitterMs: number | null;
} {
  if (pairs.length === 0) return { meanLatencyMs: null, jitterMs: null };
  const latencies = pairs.map((p) => p.latencyMs);
  const mean = latencies.reduce((a, b) => a + b, 0) / latencies.length;
  const jitter =
    latencies.length > 1
      ? Math.sqrt(latencies.reduce((s, l) => s + (l - mean) ** 2, 0) / latencies.length)
      : null;
  return { meanLatencyMs: mean, jitterMs: jitter };
}
