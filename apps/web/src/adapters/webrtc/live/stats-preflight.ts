import {
  collectNormalizedStats,
  hasRtpAudioCounterAvailability,
  type NormalizedStatsSample,
} from "./stats-sampler";
import { isObservedCategory, RTP_AUDIO_COUNTER_SOURCE } from "./types";

export type StatsPreflightState = "idle" | "probing" | "available" | "exhausted" | "cancelled";

export interface SanitizedReportShape {
  report_type: string;
  kind: string | null;
  mediaType: string | null;
  has_codec_id: boolean;
  codec_mime_family: string | null;
  has_packets_received: boolean;
  has_packets_sent: boolean;
  has_bytes_received: boolean;
  has_bytes_sent: boolean;
}

export interface RtpPreflightDiagnostics {
  state: StatsPreflightState;
  attempts: number;
  inbound_audio_seen: boolean;
  outbound_audio_seen: boolean;
  elapsed_ms: number;
  failure_reason: string | null;
  sanitized_report_shapes: SanitizedReportShape[];
}

export const DEFAULT_RTP_PREFLIGHT_CONFIG = {
  intervalMs: 500,
  maximumDurationMs: 30_000,
  maximumAttempts: 60,
} as const;

export interface RtpStatsPreflightConfig {
  intervalMs: number;
  maximumDurationMs: number;
  maximumAttempts: number;
}

export interface RtpStatsPreflightHooks {
  now?: () => number;
  scheduleRetry?: (delayMs: number, callback: () => void) => ReturnType<typeof setTimeout>;
  clearRetry?: (handle: ReturnType<typeof setTimeout>) => void;
  onUpdate?: () => void;
}

function defaultNow(): number {
  return performance.now();
}

function defaultScheduleRetry(delayMs: number, callback: () => void): ReturnType<typeof setTimeout> {
  return setTimeout(callback, delayMs);
}

function defaultClearRetry(handle: ReturnType<typeof setTimeout>): void {
  clearTimeout(handle);
}

export function sanitizeStatsReportShapes(stats: RTCStatsReport): SanitizedReportShape[] {
  const codecMime = new Map<string, string>();
  stats.forEach((report) => {
    const record = report as unknown as Record<string, unknown>;
    if (record.type === "codec" && typeof record.mimeType === "string") {
      codecMime.set(String(record.id), record.mimeType);
    }
  });

  const shapes: SanitizedReportShape[] = [];
  stats.forEach((report) => {
    const record = report as unknown as Record<string, unknown>;
    let codecMimeFamily: string | null = null;
    if (record.codecId !== undefined && record.codecId !== null) {
      const mime = codecMime.get(String(record.codecId));
      if (mime?.startsWith("audio/")) codecMimeFamily = "audio";
      else if (mime?.startsWith("video/")) codecMimeFamily = "video";
    }
    shapes.push({
      report_type: typeof record.type === "string" ? record.type : "unknown",
      kind: typeof record.kind === "string" ? record.kind : null,
      mediaType: typeof record.mediaType === "string" ? record.mediaType : null,
      has_codec_id: record.codecId !== undefined && record.codecId !== null,
      codec_mime_family: codecMimeFamily,
      has_packets_received: typeof record.packetsReceived === "number",
      has_packets_sent: typeof record.packetsSent === "number",
      has_bytes_received: typeof record.bytesReceived === "number",
      has_bytes_sent: typeof record.bytesSent === "number",
    });
  });
  return shapes;
}

function hasInboundRtpAudio(sample: NormalizedStatsSample): boolean {
  return isObservedCategory(sample.inboundAudio.counterSource, RTP_AUDIO_COUNTER_SOURCE);
}

function hasOutboundRtpAudio(sample: NormalizedStatsSample): boolean {
  return isObservedCategory(sample.outboundAudio.counterSource, RTP_AUDIO_COUNTER_SOURCE);
}

export class RtpStatsPreflightController {
  private state: StatsPreflightState = "idle";
  private generation = 0;
  private attemptCount = 0;
  private firstAttemptAt: number | null = null;
  private lastAttemptAt: number | null = null;
  private inboundAudioSeen = false;
  private outboundAudioSeen = false;
  private failureReason: string | null = null;
  private sanitizedShapes: SanitizedReportShape[] = [];
  private retryHandle: ReturnType<typeof setTimeout> | null = null;
  private inFlight = false;
  private inFlightGeneration: number | null = null;
  private restartRequestedForGeneration: number | null = null;
  private readonly now: () => number;
  private readonly scheduleRetry: (delayMs: number, callback: () => void) => ReturnType<typeof setTimeout>;
  private readonly clearRetry: (handle: ReturnType<typeof setTimeout>) => void;
  private readonly onUpdate: () => void;

  constructor(
    private readonly pc: RTCPeerConnection,
    private readonly config: RtpStatsPreflightConfig = DEFAULT_RTP_PREFLIGHT_CONFIG,
    hooks: RtpStatsPreflightHooks = {},
  ) {
    this.now = hooks.now ?? defaultNow;
    this.scheduleRetry = hooks.scheduleRetry ?? defaultScheduleRetry;
    this.clearRetry = hooks.clearRetry ?? defaultClearRetry;
    this.onUpdate = hooks.onUpdate ?? (() => {});
  }

  getState(): StatsPreflightState {
    return this.state;
  }

  get inFlightCall(): boolean {
    return this.inFlight;
  }

  getDiagnostics(): RtpPreflightDiagnostics {
    const elapsed =
      this.firstAttemptAt === null ? 0 : Math.max(0, this.elapsedSinceStart());
    return {
      state: this.state,
      attempts: this.attemptCount,
      inbound_audio_seen: this.inboundAudioSeen,
      outbound_audio_seen: this.outboundAudioSeen,
      elapsed_ms: Math.round(elapsed),
      failure_reason: this.failureReason,
      sanitized_report_shapes: this.sanitizedShapes,
    };
  }

  maybeStart(): void {
    if (this.state === "available" || this.state === "exhausted") return;
    if (this.state === "probing" && !this.inFlight) return;
    if (this.inFlight) {
      this.requestRestartWhenIdle();
      return;
    }
    if (this.state === "idle") {
      this.startNewGeneration();
    }
  }

  invalidate(reason: string): void {
    this.clearPendingRetry();
    this.generation += 1;
    this.clearRestartRequest();
    this.state = "cancelled";
    this.failureReason = reason;
    this.resetGenerationObservations();
    this.onUpdate();
  }

  restartWhenIdle(): void {
    if (this.state === "available") return;
    if (this.inFlight) {
      this.requestRestartWhenIdle();
      return;
    }
    if (this.state === "cancelled" || this.state === "idle") {
      this.startNewGeneration();
    }
  }

  reset(): void {
    this.clearPendingRetry();
    this.generation += 1;
    this.state = "idle";
    this.failureReason = null;
    this.resetGenerationObservations();
    this.clearRestartRequest();
    this.onUpdate();
  }

  /** @deprecated use invalidate */
  cancel(reason = "cancelled"): void {
    this.invalidate(reason);
  }

  private startNewGeneration(): void {
    this.beginGeneration();
    this.state = "probing";
    this.failureReason = null;
    void this.runAttempt();
  }

  private beginGeneration(): void {
    this.generation += 1;
    this.resetGenerationObservations();
    this.clearPendingRetry();
  }

  private resetGenerationObservations(): void {
    this.attemptCount = 0;
    this.firstAttemptAt = null;
    this.lastAttemptAt = null;
    this.inboundAudioSeen = false;
    this.outboundAudioSeen = false;
    this.sanitizedShapes = [];
  }

  private elapsedSinceStart(): number {
    if (this.firstAttemptAt === null) return 0;
    return this.now() - this.firstAttemptAt;
  }

  private clearPendingRetry(): void {
    if (this.retryHandle !== null) {
      this.clearRetry(this.retryHandle);
      this.retryHandle = null;
    }
  }

  private markAvailable(): void {
    this.state = "available";
    this.failureReason = null;
    this.clearPendingRetry();
    this.onUpdate();
  }

  private markExhausted(reason: string): void {
    this.state = "exhausted";
    this.failureReason = reason;
    this.clearPendingRetry();
    this.onUpdate();
  }

  private scheduleNextAttempt(): void {
    if (this.state !== "probing") return;
    this.clearPendingRetry();
    const token = this.generation;
    this.retryHandle = this.scheduleRetry(this.config.intervalMs, () => {
      if (token !== this.generation || this.state !== "probing") return;
      void this.runAttempt();
    });
  }

  private shouldContinueProbing(): boolean {
    if (this.attemptCount >= this.config.maximumAttempts) {
      return false;
    }
    if (this.firstAttemptAt !== null && this.elapsedSinceStart() >= this.config.maximumDurationMs) {
      return false;
    }
    return true;
  }

  private clearRestartRequest(): void {
    this.restartRequestedForGeneration = null;
  }

  private requestRestartWhenIdle(): void {
    this.restartRequestedForGeneration = this.generation;
  }

  private finishInFlight(token: number): void {
    if (this.inFlightGeneration !== token) return;
    this.inFlight = false;
    this.inFlightGeneration = null;
    if (
      this.restartRequestedForGeneration !== null &&
      this.restartRequestedForGeneration === this.generation &&
      (this.state === "cancelled" || this.state === "idle")
    ) {
      this.clearRestartRequest();
      this.startNewGeneration();
    }
  }

  private async runAttempt(): Promise<void> {
    if (this.state !== "probing") return;
    if (this.inFlight) return;

    const token = this.generation;
    this.inFlight = true;
    this.inFlightGeneration = token;
    const attemptStartedAt = this.now();
    if (this.firstAttemptAt === null) {
      this.firstAttemptAt = attemptStartedAt;
    }

    try {
      const stats = await this.pc.getStats();
      if (token !== this.generation || this.state !== "probing") return;

      this.sanitizedShapes = sanitizeStatsReportShapes(stats);
      const sample = await collectNormalizedStats(
        this.pc,
        this.elapsedSinceStart(),
        null,
        stats,
      );
      if (token !== this.generation || this.state !== "probing") return;

      this.attemptCount += 1;
      this.lastAttemptAt = this.now();
      if (hasInboundRtpAudio(sample)) this.inboundAudioSeen = true;
      if (hasOutboundRtpAudio(sample)) this.outboundAudioSeen = true;

      if (hasRtpAudioCounterAvailability(sample) || (this.inboundAudioSeen && this.outboundAudioSeen)) {
        this.markAvailable();
        return;
      }

      if (!this.shouldContinueProbing()) {
        this.markExhausted("bounded RTP preflight window exhausted without genuine audio RTP counters");
        return;
      }

      this.onUpdate();
      this.scheduleNextAttempt();
    } catch {
      if (token !== this.generation || this.state !== "probing") return;
      this.attemptCount += 1;
      this.lastAttemptAt = this.now();
      if (!this.shouldContinueProbing()) {
        this.markExhausted("bounded RTP preflight window exhausted after getStats failures");
        return;
      }
      this.onUpdate();
      this.scheduleNextAttempt();
    } finally {
      this.finishInFlight(token);
    }
  }
}
