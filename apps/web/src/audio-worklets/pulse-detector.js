class PulseDetectorProcessor extends AudioWorkletProcessor {
  constructor() {
    super();
    this.threshold = 0.1;
    this.lastDetection = 0;
    this.cooldownSamples = 2205;
  }

  process(inputs) {
    const input = inputs[0];
    if (!input || !input[0]) return true;

    const channel = input[0];
    let peak = 0;
    for (let i = 0; i < channel.length; i++) {
      const abs = Math.abs(channel[i]);
      if (abs > peak) peak = abs;
    }

    if (peak > this.threshold && currentFrame - this.lastDetection > this.cooldownSamples) {
      this.lastDetection = currentFrame;
      this.port.postMessage({ type: "pulse_detected", timestamp: currentTime * 1000 });
    }

    return true;
  }
}

registerProcessor("pulse-detector", PulseDetectorProcessor);
