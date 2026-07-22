export const LIVE_SCHEMA_VERSION = "echlub.performance.live-observation/v1";
export const DATA_CHANNEL_LABEL = "echlub-clock-probe";
export const DEFAULT_PROBE_INTERVAL_MS = 1000;
export const DEFAULT_PROBE_COUNT = 30;
export const MAX_PROBE_COUNT = 120;
export const DEFAULT_STATS_INTERVAL_MS = 1000;
export const DEFAULT_OBSERVATION_SECONDS = 60;
export const MAX_OBSERVATION_SECONDS = 180;
export const MAX_STATS_SAMPLES = 180;

export type PeerRole = "peer_a" | "peer_b";
export type CaptureProfile = "browser_default" | "music_low_latency";

export type LiveSessionPhase =
  | "idle"
  | "prepared"
  | "microphone_ready"
  | "signaling_connecting"
  | "peer_present"
  | "negotiating"
  | "connected"
  | "ready_to_observe"
  | "observing"
  | "finalizing"
  | "completed"
  | "failed"
  | "stopped";

export type MetricValue =
  | { kind: "observed_number"; value: number }
  | { kind: "observed_boolean"; value: boolean }
  | { kind: "observed_category"; value: string }
  | { kind: "unsupported" }
  | { kind: "unavailable"; reason: string }
  | { kind: "invalid"; reason: string };

export interface ClockProbeSample {
  sequence: number;
  requesterRole: PeerRole;
  responderRole: PeerRole | null;
  protocolVersion: number;
  t0: number;
  t1: number | null;
  t2: number | null;
  t3: number | null;
  rttMs: number | null;
  offsetMs: number | null;
  timeout: boolean;
  duplicate: boolean;
  unsolicited: boolean;
  invalid: boolean;
}

export interface LiveSessionConfig {
  sessionCorrelationId: string;
  localPeerId: PeerRole;
  signalingUrl?: string;
  softwareCommit?: string;
  captureProfile?: CaptureProfile;
}

export interface ObservationConfig {
  durationSeconds?: number;
  statsIntervalMs?: number;
  probeIntervalMs?: number;
  probeCount?: number;
}

export interface LiveSessionCallbacks {
  onPhaseChange: (phase: LiveSessionPhase) => void;
  onSignalingState: (state: string) => void;
  onConnectionState: (state: RTCPeerConnectionState) => void;
  onIceState: (state: RTCIceConnectionState) => void;
  onDataChannelState: (state: RTCDataChannelState) => void;
  onRemoteStream: (stream: MediaStream) => void;
  onMicrophoneState: (active: boolean) => void;
  onNegotiation: (event: string) => void;
  onClockProbe: (sample: ClockProbeSample) => void;
  onStatsSample: (sample: unknown) => void;
  onError: (message: string) => void;
}

export interface SignalingEnvelope {
  type: "description" | "ice_candidate" | "control";
  descriptionType?: RTCSdpType;
  sdp?: RTCSessionDescriptionInit;
  candidate?: RTCIceCandidateInit;
  control?: string;
}

export function generateSessionCorrelationId(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(16));
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

export const SESSION_CORRELATION_ID_PATTERN = /^[0-9a-fA-F]{32}$/;

export function normalizeSessionCorrelationId(value: string): string | null {
  const trimmed = value.trim();
  if (!SESSION_CORRELATION_ID_PATTERN.test(trimmed)) {
    return null;
  }
  return trimmed.toLowerCase();
}

export function isValidSessionCorrelationId(value: string): boolean {
  return normalizeSessionCorrelationId(value) !== null;
}

export function observedNumber(value: number): MetricValue {
  return { kind: "observed_number", value };
}

export function observedBoolean(value: boolean): MetricValue {
  return { kind: "observed_boolean", value };
}

export function observedCategory(value: string): MetricValue {
  return { kind: "observed_category", value };
}

/** @deprecated use observedNumber */
export function observed(value: number): MetricValue {
  return observedNumber(value);
}

export function unsupported(): MetricValue {
  return { kind: "unsupported" };
}

export function unavailable(reason: string): MetricValue {
  return { kind: "unavailable", reason };
}

export function invalid(reason: string): MetricValue {
  return { kind: "invalid", reason };
}

export function isObservedNumber(
  metric: MetricValue,
): metric is { kind: "observed_number"; value: number } {
  return metric.kind === "observed_number";
}

export const CANDIDATE_CATEGORIES = ["host", "srflx", "prflx", "relay"] as const;
export type CandidateCategory = (typeof CANDIDATE_CATEGORIES)[number];

export const CONNECTION_STATE_CATEGORIES = ["succeeded", "in-progress"] as const;
export type ConnectionStateCategory = (typeof CONNECTION_STATE_CATEGORIES)[number];

export const TRANSPORT_PROTOCOL_CATEGORIES = ["udp", "tcp"] as const;
export type TransportProtocolCategory = (typeof TRANSPORT_PROTOCOL_CATEGORIES)[number];

export function captureConstraints(profile: CaptureProfile): MediaTrackConstraints {
  if (profile === "music_low_latency") {
    return {
      echoCancellation: false,
      noiseSuppression: false,
      autoGainControl: false,
      channelCount: 1,
    };
  }
  return {
    echoCancellation: true,
    noiseSuppression: true,
    autoGainControl: true,
  };
}
