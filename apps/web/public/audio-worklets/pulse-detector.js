class PulseDetectorProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.lastPeakAt = 0;
    this.targetFreq = 1000;
    this.sampleRate = sampleRate;
  }

  process(inputs) {
    const input = inputs[0]?.[0];
    if (!input) return true;

    let peak = 0;
    for (let i = 0; i < input.length; i++) {
      peak = Math.max(peak, Math.abs(input[i]));
    }

    const now = currentTime * 1000;
    if (peak > 0.015 && now - this.lastPeakAt > 0.08) {
      this.lastPeakAt = now;
      this.port.postMessage({ type: "peak", peak, atMs: now, frequencyHz: this.targetFreq });
    }
    return true;
  }
}

registerProcessor("pulse-detector", PulseDetectorProcessor);
