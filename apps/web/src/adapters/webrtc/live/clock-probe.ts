import type { ClockProbeSample, PeerRole } from "./types";

export const CLOCK_PROBE_PROTOCOL_VERSION = 1;
export const MAX_PROBE_PAYLOAD_BYTES = 512;
export const PROBE_TIMEOUT_MS = 5000;
/** JSON-serialization tolerance for stored vs canonical clock metrics (milliseconds). */
export const CLOCK_METRIC_EPSILON_MS = 1e-6;

interface ProbeRequest {
  type: "clock_probe_request";
  protocolVersion: number;
  sequence: number;
  requesterRole: PeerRole;
  t0: number;
}

interface ProbeResponse {
  type: "clock_probe_response";
  protocolVersion: number;
  sequence: number;
  requesterRole: PeerRole;
  responderRole: PeerRole;
  t0: number;
  t1: number;
  t2: number;
}

export interface CrossDeviceClockValidation {
  valid: boolean;
  rttMs: number | null;
  offsetMs: number | null;
}

function utf8ByteLength(value: string): number {
  return new TextEncoder().encode(value).length;
}

export function oppositeRole(role: PeerRole): PeerRole {
  return role === "peer_a" ? "peer_b" : "peer_a";
}

export function validateCrossDeviceClockTimestamps(
  t0: number,
  t1: number,
  t2: number,
  t3: number,
): CrossDeviceClockValidation {
  if (![t0, t1, t2, t3].every(Number.isFinite)) {
    return { valid: false, rttMs: null, offsetMs: null };
  }
  if (t3 < t0 || t2 < t1) {
    return { valid: false, rttMs: null, offsetMs: null };
  }
  const remoteProcessingDuration = t2 - t1;
  const localRoundTripDuration = t3 - t0;
  if (remoteProcessingDuration < 0 || localRoundTripDuration < 0) {
    return { valid: false, rttMs: null, offsetMs: null };
  }
  const rtt = localRoundTripDuration - remoteProcessingDuration;
  if (!Number.isFinite(rtt) || rtt < 0) {
    return { valid: false, rttMs: null, offsetMs: null };
  }
  const offset = (t1 - t0 + (t2 - t3)) / 2;
  return {
    valid: true,
    rttMs: rtt,
    offsetMs: Number.isFinite(offset) ? offset : null,
  };
}

export function verifyStoredClockMetrics(
  sample: Pick<ClockProbeSample, "t0" | "t1" | "t2" | "t3" | "rttMs" | "offsetMs">,
): boolean {
  if (
    sample.t0 === null ||
    sample.t1 === null ||
    sample.t2 === null ||
    sample.t3 === null ||
    sample.rttMs === null
  ) {
    return false;
  }
  if (!Number.isFinite(sample.rttMs) || sample.rttMs < 0) {
    return false;
  }
  const validation = validateCrossDeviceClockTimestamps(
    sample.t0,
    sample.t1,
    sample.t2,
    sample.t3,
  );
  if (!validation.valid || validation.rttMs === null) {
    return false;
  }
  if (Math.abs(sample.rttMs - validation.rttMs) > CLOCK_METRIC_EPSILON_MS) {
    return false;
  }
  if (sample.offsetMs !== null) {
    if (!Number.isFinite(sample.offsetMs) || validation.offsetMs === null) {
      return false;
    }
    if (Math.abs(sample.offsetMs - validation.offsetMs) > CLOCK_METRIC_EPSILON_MS) {
      return false;
    }
  }
  return true;
}

export class ClockProbeEngine {
  private channel: RTCDataChannel | null = null;
  private readonly onSample: (sample: ClockProbeSample) => void;
  private readonly localRole: PeerRole;
  private intervalId: ReturnType<typeof setInterval> | null = null;
  private pending = new Map<string, { t0: number; timeoutId: ReturnType<typeof setTimeout> }>();
  private sentSequences = new Set<string>();
  private respondedSequences = new Set<string>();
  private completedResponses = new Set<string>();
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
    this.completedResponses.clear();
  }

  private sequenceKey(role: PeerRole, sequence: number): string {
    return `${role}:${sequence}`;
  }

  private invalidSample(
    sequence: number,
    requesterRole: PeerRole,
    responderRole: PeerRole | null,
    protocolVersion: number,
    t0: number,
    t1: number | null = null,
    t2: number | null = null,
    t3: number | null = null,
    flags: Partial<Pick<ClockProbeSample, "duplicate" | "unsolicited">> = {},
  ): ClockProbeSample {
    return {
      sequence,
      requesterRole,
      responderRole,
      protocolVersion,
      t0,
      t1,
      t2,
      t3,
      rttMs: null,
      offsetMs: null,
      timeout: false,
      duplicate: flags.duplicate ?? false,
      unsolicited: flags.unsolicited ?? false,
      invalid: true,
    };
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
      requesterRole: this.localRole,
      t0,
    };
    const encoded = JSON.stringify(payload);
    if (utf8ByteLength(encoded) > MAX_PROBE_PAYLOAD_BYTES) return;

    if (this.sentSequences.has(key)) {
      this.onSample(
        this.invalidSample(
          sequence,
          this.localRole,
          null,
          CLOCK_PROBE_PROTOCOL_VERSION,
          t0,
          null,
          null,
          null,
          { duplicate: true },
        ),
      );
      return;
    }
    this.sentSequences.add(key);

    const timeoutId = setTimeout(() => {
      this.pending.delete(key);
      this.onSample({
        sequence,
        requesterRole: this.localRole,
        responderRole: null,
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
        const requesterRole =
          "requesterRole" in msg && msg.requesterRole ? msg.requesterRole : this.localRole;
        const responderRole =
          "responderRole" in msg && msg.responderRole ? msg.responderRole : null;
        this.onSample(
          this.invalidSample(
            "sequence" in msg ? msg.sequence : -1,
            requesterRole,
            responderRole,
            msg.protocolVersion ?? 0,
            "t0" in msg && Number.isFinite(msg.t0) ? msg.t0 : 0,
          ),
        );
        return;
      }

      if (msg.type === "clock_probe_request" && "t0" in msg) {
        if (msg.requesterRole === this.localRole) return;
        if (msg.requesterRole !== oppositeRole(this.localRole)) return;
        if (!Number.isFinite(msg.t0)) return;

        const key = this.sequenceKey(msg.requesterRole, msg.sequence);
        if (this.respondedSequences.has(key)) return;
        this.respondedSequences.add(key);
        const t1 = performance.timeOrigin + performance.now();
        const t2 = performance.timeOrigin + performance.now();
        const response: ProbeResponse = {
          type: "clock_probe_response",
          protocolVersion: CLOCK_PROBE_PROTOCOL_VERSION,
          sequence: msg.sequence,
          requesterRole: msg.requesterRole,
          responderRole: this.localRole,
          t0: msg.t0,
          t1,
          t2,
        };
        const encoded = JSON.stringify(response);
        if (utf8ByteLength(encoded) > MAX_PROBE_PAYLOAD_BYTES) return;
        this.channel?.send(encoded);
      } else if (msg.type === "clock_probe_response") {
        const { requesterRole, responderRole, sequence, t0, t1, t2 } = msg;

        if (requesterRole !== this.localRole) {
          this.onSample(
            this.invalidSample(
              sequence,
              requesterRole,
              responderRole,
              msg.protocolVersion,
              t0,
              t1,
              t2,
              null,
            ),
          );
          return;
        }

        if (responderRole !== oppositeRole(this.localRole)) {
          this.onSample(
            this.invalidSample(
              sequence,
              requesterRole,
              responderRole,
              msg.protocolVersion,
              t0,
              t1,
              t2,
              null,
            ),
          );
          return;
        }

        const key = this.sequenceKey(this.localRole, sequence);
        const responseKey = `${key}:response`;
        if (this.completedResponses.has(responseKey)) {
          this.onSample({
            sequence,
            requesterRole,
            responderRole,
            protocolVersion: msg.protocolVersion,
            t0,
            t1,
            t2,
            t3: null,
            rttMs: null,
            offsetMs: null,
            timeout: false,
            duplicate: true,
            unsolicited: false,
            invalid: false,
          });
          return;
        }

        const pending = this.pending.get(key);
        if (!pending) {
          this.onSample({
            sequence,
            requesterRole,
            responderRole,
            protocolVersion: msg.protocolVersion,
            t0,
            t1,
            t2,
            t3: null,
            rttMs: null,
            offsetMs: null,
            timeout: false,
            duplicate: false,
            unsolicited: true,
            invalid: false,
          });
          return;
        }

        clearTimeout(pending.timeoutId);
        this.pending.delete(key);
        const t3 = performance.timeOrigin + performance.now();
        const validation = validateCrossDeviceClockTimestamps(t0, t1, t2, t3);
        if (!validation.valid) {
          this.onSample(
            this.invalidSample(
              sequence,
              requesterRole,
              responderRole,
              msg.protocolVersion,
              t0,
              t1,
              t2,
              t3,
            ),
          );
          return;
        }

        this.completedResponses.add(responseKey);
        this.onSample({
          sequence,
          requesterRole,
          responderRole,
          protocolVersion: msg.protocolVersion,
          t0,
          t1,
          t2,
          t3,
          rttMs: validation.rttMs,
          offsetMs: validation.offsetMs,
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
    (s) =>
      !s.timeout &&
      !s.duplicate &&
      !s.unsolicited &&
      !s.invalid &&
      s.rttMs !== null &&
      verifyStoredClockMetrics(s),
  );
  if (valid.length === 0) return { rtt: null, offset: null, madRtt: null };
  const rtts = valid.map((s) => s.rttMs!).sort((a, b) => a - b);
  const offsets = valid
    .map((s) => s.offsetMs)
    .filter((v): v is number => v !== null)
    .sort((a, b) => a - b);
  const mid = Math.floor(rtts.length / 2);
  const rtt =
    rtts.length % 2 === 0 ? (rtts[mid - 1]! + rtts[mid]!) / 2 : rtts[mid]!;
  const offset =
    offsets.length === 0
      ? null
      : offsets.length % 2 === 0
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
    (s) =>
      !s.timeout &&
      !s.duplicate &&
      !s.unsolicited &&
      !s.invalid &&
      s.rttMs !== null &&
      verifyStoredClockMetrics(s),
  );
}

export function validLocalCompletedProbes(
  samples: ClockProbeSample[],
  localRole: PeerRole,
): ClockProbeSample[] {
  return validCompletedProbes(samples).filter(
    (s) =>
      s.requesterRole === localRole && s.responderRole === oppositeRole(localRole),
  );
}
