import type { ClockProbeSample } from "./types";

export const CLOCK_PROBE_PROTOCOL_VERSION = 1;
export const MAX_PROBE_PAYLOAD_BYTES = 512;
export const PROBE_TIMEOUT_MS = 5000;

interface ProbeRequest {
  type: "clock_probe_request";
  protocolVersion: number;
  sequence: number;
  t0: number;
}

interface ProbeResponse {
  type: "clock_probe_response";
  protocolVersion: number;
  sequence: number;
  t0: number;
  t1: number;
  t2: number;
}

export class ClockProbeEngine {
  private channel: RTCDataChannel | null = null;
  private readonly onSample: (sample: ClockProbeSample) => void;
  private readonly isInitiator: boolean;
  private intervalId: ReturnType<typeof setInterval> | null = null;
  private pending = new Map<number, { t0: number; timeoutId: ReturnType<typeof setTimeout> }>();
  private seen = new Set<number>();
  private nextSequence = 0;
  private config = { intervalMs: 1000, count: 30 };

  constructor(isInitiator: boolean, onSample: (sample: ClockProbeSample) => void) {
    this.isInitiator = isInitiator;
    this.onSample = onSample;
  }

  attach(channel: RTCDataChannel): void {
    this.channel = channel;
    channel.onmessage = (event) => this.handleMessage(event.data as string);
  }

  start(config: { intervalMs: number; count: number }): void {
    this.stop();
    this.config = {
      intervalMs: config.intervalMs,
      count: Math.min(config.count, 120),
    };
    if (!this.isInitiator) return;
    let sent = 0;
    this.intervalId = setInterval(() => {
      if (sent >= this.config.count) {
        this.stopProbes();
        return;
      }
      this.sendProbe();
      sent += 1;
    }, this.config.intervalMs);
    this.sendProbe();
    sent += 1;
  }

  stopProbes(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = null;
    }
    for (const [, pending] of this.pending) {
      clearTimeout(pending.timeoutId);
    }
    this.pending.clear();
  }

  stop(): void {
    this.stopProbes();
    this.channel = null;
    this.seen.clear();
  }

  private sendProbe(): void {
    if (!this.channel || this.channel.readyState !== "open") return;
    const sequence = this.nextSequence++;
    const t0 = performance.timeOrigin + performance.now();
    const payload: ProbeRequest = {
      type: "clock_probe_request",
      protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
      sequence,
      t0,
    };
    const encoded = JSON.stringify(payload);
    if (encoded.length > MAX_PROBE_PAYLOAD_BYTES) return;
    if (this.seen.has(sequence)) {
      this.onSample({
        sequence,
        protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
        t0,
        t1: null,
        t2: null,
        t3: null,
        rttMs: null,
        offsetMs: null,
        timeout: false,
        duplicate: true,
      });
      return;
    }
    this.seen.add(sequence);
    const timeoutId = setTimeout(() => {
      this.pending.delete(sequence);
      this.onSample({
        sequence,
        protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
        t0,
        t1: null,
        t2: null,
        t3: null,
        rttMs: null,
        offsetMs: null,
        timeout: true,
        duplicate: false,
      });
    }, PROBE_TIMEOUT_MS);
    this.pending.set(sequence, { t0, timeoutId });
    this.channel.send(encoded);
  }

  private handleMessage(raw: string): void {
    try {
      const msg = JSON.parse(raw) as ProbeRequest | ProbeResponse;
      if (msg.type === "clock_probe_request" && "t0" in msg) {
        const t1 = performance.timeOrigin + performance.now();
        const t2 = performance.timeOrigin + performance.now();
        const response: ProbeResponse = {
          type: "clock_probe_response",
          protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
          sequence: msg.sequence,
          t0: msg.t0,
          t1,
          t2,
        };
        this.channel?.send(JSON.stringify(response));
      } else if (msg.type === "clock_probe_response") {
        const t3 = performance.timeOrigin + performance.now();
        const pending = this.pending.get(msg.sequence);
        if (pending) {
          clearTimeout(pending.timeoutId);
          this.pending.delete(msg.sequence);
        }
        const rtt = t3 - msg.t0 - (msg.t2 - msg.t1);
        const offset = (msg.t1 - msg.t0 + (msg.t2 - t3)) / 2;
        if (rtt < 0) return;
        this.onSample({
          sequence: msg.sequence,
          protocolVersion: msg.protocolVersion,
          t0: msg.t0,
          t1: msg.t1,
          t2: msg.t2,
          t3,
          rttMs: rtt,
          offsetMs: offset,
          timeout: false,
          duplicate: false,
        });
      }
    } catch {
      /* ignore malformed */
    }
  }
}

export function computeClockMedian(samples: ClockProbeSample[]): { rtt: number | null; offset: number | null } {
  const valid = samples.filter((s) => !s.timeout && !s.duplicate && s.rttMs !== null);
  if (valid.length === 0) return { rtt: null, offset: null };
  const rtts = valid.map((s) => s.rttMs!).sort((a, b) => a - b);
  const offsets = valid.map((s) => s.offsetMs ?? 0).sort((a, b) => a - b);
  const mid = Math.floor(rtts.length / 2);
  const rtt =
    rtts.length % 2 === 0 ? (rtts[mid - 1]! + rtts[mid]!) / 2 : rtts[mid]!;
  const offset =
    offsets.length % 2 === 0
      ? (offsets[mid - 1]! + offsets[mid]!) / 2
      : offsets[mid]!;
  return { rtt, offset };
}
