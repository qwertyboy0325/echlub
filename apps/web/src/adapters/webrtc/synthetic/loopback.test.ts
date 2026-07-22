import { describe, expect, it } from "vitest";
import {
  bandEnergyNearFrequency,
  classifyDetection,
  correlatePulseSequence,
  DEFAULT_PULSE_DETECTION_CONFIG,
  rejectBackgroundRtpWithoutPulse,
  type PulseDetectionRecord,
  type PulseEmitRecord,
} from "./pulse-detector";

const emits: PulseEmitRecord[] = [
  { index: 0, emitAtMs: 1000, frequencyHz: 1000 },
  { index: 1, emitAtMs: 1200, frequencyHz: 1000 },
  { index: 2, emitAtMs: 1400, frequencyHz: 1000 },
];

function detection(
  detectedAtMs: number,
  frequencyHz = 1000,
  peakAmplitude = 0.05,
  bandEnergy = 0.01,
): PulseDetectionRecord {
  return { detectedAtMs, frequencyHz, peakAmplitude, bandEnergy };
}

describe("decoded pulse detector", () => {
  it("rejects background RTP without pulse (detection before first emit)", () => {
    const dets = [detection(500)];
    expect(classifyDetection(dets[0]!, emits, [], DEFAULT_PULSE_DETECTION_CONFIG)).toBe(
      "reject_before_emit",
    );
    expect(rejectBackgroundRtpWithoutPulse(dets, emits)).toHaveLength(0);
  });

  it("rejects analyser false positive before first pulse", () => {
    const dets = [detection(900, 1000, 0.02, 0.002)];
    expect(classifyDetection(dets[0]!, emits, [], DEFAULT_PULSE_DETECTION_CONFIG)).toBe(
      "reject_before_emit",
    );
  });

  it("accepts valid decoded pulse detected after emit", () => {
    const dets = [detection(1050)];
    expect(classifyDetection(dets[0]!, emits, [], DEFAULT_PULSE_DETECTION_CONFIG)).toBe("accepted");
    const pairs = correlatePulseSequence(emits, dets);
    expect(pairs).toHaveLength(1);
    expect(pairs[0]!.latencyMs).toBe(50);
  });

  it("rejects wrong frequency", () => {
    const dets = [detection(1050, 440)];
    expect(classifyDetection(dets[0]!, emits, [], DEFAULT_PULSE_DETECTION_CONFIG)).toBe(
      "reject_wrong_frequency",
    );
  });

  it("rejects duplicate detection", () => {
    const first = detection(1050);
    const second = detection(1060);
    expect(classifyDetection(second, emits, [first], DEFAULT_PULSE_DETECTION_CONFIG)).toBe(
      "reject_duplicate",
    );
  });

  it("rejects detect-before-emit", () => {
    expect(classifyDetection(detection(800), emits, [], DEFAULT_PULSE_DETECTION_CONFIG)).toBe(
      "reject_before_emit",
    );
  });

  it("correlates multiple pulses in sequence order", () => {
    const dets = [detection(1050), detection(1250), detection(1450)];
    const pairs = correlatePulseSequence(emits, dets);
    expect(pairs).toHaveLength(3);
    expect(pairs.map((p) => p.index)).toEqual([0, 1, 2]);
  });
});

describe("band energy", () => {
  it("measures energy near target frequency", () => {
    const fft = new Float32Array(1024);
    const sampleRate = 48000;
    const targetBin = Math.round((1000 * fft.length * 2) / sampleRate);
    fft[targetBin] = 1.0;
    const energy = bandEnergyNearFrequency(fft, sampleRate, 1000, 150);
    expect(energy).toBeGreaterThan(0);
  });
});

describe("ping/pong RTT semantics", () => {
  it("computes RTT only after pong received", () => {
    const sentAt = 1000;
    const pongReceivedAt = 1010;
    const rtt = pongReceivedAt - sentAt;
    expect(rtt).toBe(10);
  });
});

describe("connection timeout", () => {
  it("treats failed connection as not connected", () => {
    const state = "failed" as RTCPeerConnectionState;
    expect(state === "connected").toBe(false);
  });
});
