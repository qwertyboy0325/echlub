import type { ClockProbeSample, PeerRole } from "./types";

export const CLOCK_PROBE_PROTOCOL_VERSION = 1;
export const MAX_PROBE_PAYLOAD_BYTES = 512;
export const PROBE_TIMEOUT_MS = 5000;

interface ProbeRequest {
  type: "clock_probe_request";
  protocolVersion: number;
  sequence: number;
  senderRole: PeerRole;
  t0: number;
}

interface ProbeResponse {
  type: "clock_probe_response";
  protocolVersion: number;
  sequence: number;
  senderRole: PeerRole;
  responderRole: PeerRole;
  t0: number;
  t1: number;
  t2: number;
}

export class ClockProbeEngine {
  private channel: RTCDataChannel | null = null;
  private readonly onSample: (sample: ClockProbeSample) => void;
  private readonly localRole: PeerRole;
  private intervalId: ReturnType<typeof setInterval> | null = null;
  private pending = new Map<string, { t0: number; timeoutId: ReturnType<typeof setTimeout> }>();
  private sentSequences = new Set<string>();
  private respondedSequences = new Set<string>();
  private nextSequence = 0;
  private config = { intervalMs: 1000, count: 30 };

  constructor(localRole: PeerRole, onSample: (sample: ClockProbeSample) => void) {
    this.localRole = localRole;
    this.onSample = onSample;
  }

  attach(channel: RTCDataChannel): void {
    this.channel = channel;
    channel.onmessage = (event) => this.handleMessage(event.data as string);
  }

  start(config: { intervalMs: number; count: number }): void {
    this.stopProbes();
    this.config = {
      intervalMs: config.intervalMs,
      count: Math.min(config.count, 120),
    };
    if (this.config.count === 0) return;

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
    this.sentSequences.clear();
    this.respondedSequences.clear();
  }

  private sequenceKey(role: PeerRole, sequence: number): string {
    return `${role}:${sequence}`;
  }

  private sendProbe(): void {
    if (!this.channel || this.channel.readyState !== "open") return;
    const sequence = this.nextSequence++;
    const key = this.sequenceKey(this.localRole, sequence);
    const t0 = performance.timeOrigin + performance.now();
    const payload: ProbeRequest = {
      type: "clock_probe_request",
      protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
      sequence,
      senderRole: this.localRole,
      t0,
    };
    const encoded = JSON.stringify(payload);
    if (encoded.length > MAX_PROBE_PAYLOAD_BYTES) return;

    if (this.sentSequences.has(key)) {
      this.onSample({
        sequence,
        senderRole: this.localRole,
        protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
        t0,
        t1: null,
        t2: null,
        t3: null,
        rttMs: null,
        offsetMs: null,
        timeout: false,
        duplicate: true,
        unsolicited: false,
        invalid: true,
      });
      return;
    }
    this.sentSequences.add(key);

    const timeoutId = setTimeout(() => {
      this.pending.delete(key);
      this.onSample({
        sequence,
        senderRole: this.localRole,
        protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
        t0,
        t1: null,
        t2: null,
        t3: null,
        rttMs: null,
        offsetMs: null,
        timeout: true,
        duplicate: false,
        unsolicited: false,
        invalid: false,
      });
    }, PROBE_TIMEOUT_MS);
    this.pending.set(key, { t0, timeoutId });
    this.channel.send(encoded);
  }

  private handleMessage(raw: string): void {
    try {
      const msg = JSON.parse(raw) as ProbeRequest | ProbeResponse;
      if (msg.protocolVersion !== CLOCK_PROBE_PROTOCOL_VERSION) {
        this.onSample({
          sequence: "sequence" in msg ? msg.sequence : -1,
          senderRole: "senderRole" in msg ? msg.senderRole : this.localRole,
          protocolVersion: msg.protocolVersion ?? 0,
          t0: "t0" in msg ? msg.t0 : 0,
          t1: null,
          t2: null,
          t3: null,
          rttMs: null,
          offsetMs: null,
          timeout: false,
          duplicate: false,
          unsolicited: false,
          invalid: true,
        });
        return;
      }

      if (msg.type === "clock_probe_request" && "t0" in msg) {
        const key = this.sequenceKey(msg.senderRole, msg.sequence);
        if (this.respondedSequences.has(key)) return;
        this.respondedSequences.add(key);
        const t1 = performance.timeOrigin + performance.now();
        const t2 = performance.timeOrigin + performance.now();
        const response: ProbeResponse = {
          type: "clock_probe_response",
          protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
          sequence: msg.sequence,
          senderRole: msg.senderRole,
          responderRole: this.localRole,
          t0: msg.t0,
          t1,
          t2,
        };
        this.channel?.send(JSON.stringify(response));
      } else if (msg.type === "clock_probe_response") {
        if (msg.senderRole === this.localRole) {
          this.onSample({
            sequence: msg.sequence,
            senderRole: msg.senderRole,
            protocolVersion: msg.protocolVersion,
            t0: msg.t0,
            t1: msg.t1,
            t2: msg.t2,
            t3: null,
            rttMs: null,
            offsetMs: null,
            timeout: false,
            duplicate: false,
            unsolicited: true,
            invalid: true,
          });
          return;
        }

        const key = this.sequenceKey(msg.senderRole, msg.sequence);
        const pending = this.pending.get(key);
        if (!pending) {
          this.onSample({
            sequence: msg.sequence,
            senderRole: msg.senderRole,
            protocolVersion: msg.protocolVersion,
            t0: msg.t0,
            t1: msg.t1,
            t2: msg.t2,
            t3: null,
            rttMs: null,
            offsetMs: null,
            timeout: false,
            duplicate: false,
            unsolicited: true,
            invalid: true,
          });
          return;
        }

        if (this.respondedSequences.has(`${key}:local`)) {
          this.onSample({
            sequence: msg.sequence,
            senderRole: msg.senderRole,
            protocolVersion: msg.protocolVersion,
            t0: msg.t0,
            t1: msg.t1,
            t2: msg.t2,
            t3: null,
            rttMs: null,
            offsetMs: null,
            timeout: false,
            duplicate: true,
            unsolicited: false,
            invalid: true,
          });
          return;
        }
        this.respondedSequences.add(`${key}:local`);

        clearTimeout(pending.timeoutId);
        this.pending.delete(key);
        const t3 = performance.timeOrigin + performance.now();
        const rtt = t3 - msg.t0 - (msg.t2 - msg.t1);
        const offset = (msg.t1 - msg.t0 + (msg.t2 - t3)) / 2;
        if (rtt < 0) {
          this.onSample({
            sequence: msg.sequence,
            senderRole: msg.senderRole,
            protocolVersion: msg.protocolVersion,
            t0: msg.t0,
            t1: msg.t1,
            t2: msg.t2,
            t3,
            rttMs: null,
            offsetMs: null,
            timeout: false,
            duplicate: false,
            unsolicited: false,
            invalid: true,
          });
          return;
        }
        this.onSample({
          sequence: msg.sequence,
          senderRole: msg.senderRole,
          protocolVersion: msg.protocolVersion,
          t0: msg.t0,
          t1: msg.t1,
          t2: msg.t2,
          t3,
          rttMs: rtt,
          offsetMs: offset,
          timeout: false,
          duplicate: false,
          unsolicited: false,
          invalid: false,
        });
      }
    } catch {
      /* ignore malformed */
    }
  }
}

export function computeClockMedian(samples: ClockProbeSample[]): {
  rtt: number | null;
  offset: number | null;
  madRtt: number | null;
} {
  const valid = samples.filter(
    (s) => !s.timeout && !s.duplicate && !s.unsolicited && !s.invalid && s.rttMs !== null,
  );
  if (valid.length === 0) return { rtt: null, offset: null, madRtt: null };
  const rtts = valid.map((s) => s.rttMs!).sort((a, b) => a - b);
  const offsets = valid.map((s) => s.offsetMs ?? 0).sort((a, b) => a - b);
  const mid = Math.floor(rtts.length / 2);
  const rtt =
    rtts.length % 2 === 0 ? (rtts[mid - 1]! + rtts[mid]!) / 2 : rtts[mid]!;
  const offset =
    offsets.length % 2 === 0
      ? (offsets[mid - 1]! + offsets[mid]!) / 2
      : offsets[mid]!;
  const deviations = rtts.map((v) => Math.abs(v - rtt)).sort((a, b) => a - b);
  const madRtt =
    deviations.length % 2 === 0
      ? (deviations[mid - 1]! + deviations[mid]!) / 2
      : deviations[mid]!;
  return { rtt, offset, madRtt };
}

export function validCompletedProbes(samples: ClockProbeSample[]): ClockProbeSample[] {
  return samples.filter(
    (s) => !s.timeout && !s.duplicate && !s.unsolicited && !s.invalid && s.rttMs !== null,
  );
}
