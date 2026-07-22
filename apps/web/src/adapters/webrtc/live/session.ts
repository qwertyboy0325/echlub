import {
  canonicalizeClockProbeSampleForExport,
  ClockProbeEngine,
  computeClockMedian,
  validLocalCompletedProbes,
} from "./clock-probe";
import {
  DEFAULT_RTP_PREFLIGHT_CONFIG,
  RtpStatsPreflightController,
  type RtpPreflightDiagnostics,
} from "./stats-preflight";
import {
  applySignalingDescription,
  createAndSendOffer,
  createNegotiationState,
  IceCandidateBuffer,
} from "./negotiation";
import { StatsSampler } from "./stats-sampler";
import {
  captureConstraints,
  DATA_CHANNEL_LABEL,
  DEFAULT_OBSERVATION_SECONDS,
  DEFAULT_PROBE_COUNT,
  DEFAULT_PROBE_INTERVAL_MS,
  DEFAULT_STATS_INTERVAL_MS,
  LIVE_SCHEMA_VERSION,
  MAX_OBSERVATION_SECONDS,
  MAX_STATS_SAMPLES,
  type CaptureProfile,
  type ClockProbeSample,
  type LiveSessionCallbacks,
  type LiveSessionConfig,
  type LiveSessionPhase,
  type ObservationConfig,
  type PeerRole,
  unsupported,
  unavailable,
} from "./types";

export class LiveWebRtcSession {
  private pc: RTCPeerConnection | null = null;
  private dc: RTCDataChannel | null = null;
  private audioTransceiver: RTCRtpTransceiver | null = null;
  private localStream: MediaStream | null = null;
  private remoteStream: MediaStream | null = null;
  private signaling: import("../../signaling/client").SignalingClient | null = null;
  private readonly config: LiveSessionConfig;
  private readonly callbacks: LiveSessionCallbacks;
  private readonly polite: boolean;
  private readonly negotiation = createNegotiationState(false);
  private readonly iceBuffer = new IceCandidateBuffer();
  private clockEngine: ClockProbeEngine | null = null;
  private statsSampler: StatsSampler | null = null;
  private phase: LiveSessionPhase = "idle";
  private captureProfile: CaptureProfile = "browser_default";
  private observationStartedAt: string | null = null;
  private observationCompletedAt: string | null = null;
  private clockSamples: ClockProbeSample[] = [];
  private statsSamples: unknown[] = [];
  private peerPresent = false;
  private stopped = false;
  private dataChannelProps: Record<string, unknown> = {};
  private connectionLifecycle: Record<string, unknown> = {};
  private playoutState: Record<string, unknown> = {};
  private captureState: Record<string, unknown> = {};
  private statsPreflight: RtpStatsPreflightController | null = null;
  private peerNegotiationStarted = false;

  private static readonly MIN_FINALIZED_STATS = 30;
  private static readonly MIN_FINALIZED_PROBES = 10;
  private static readonly COMMIT_SHA40 = /^[0-9a-fA-F]{40}$/;

  constructor(config: LiveSessionConfig, callbacks: LiveSessionCallbacks) {
    this.config = config;
    this.callbacks = callbacks;
    this.polite = config.localPeerId === "peer_b";
    this.negotiation.polite = this.polite;
    this.captureProfile = config.captureProfile ?? "browser_default";
  }

  prepare(): void {
    this.assertNotStopped();
    this.setPhase("prepared");
  }

  async enableMicrophone(profile?: CaptureProfile): Promise<void> {
    this.assertNotStopped();
    if (profile) this.captureProfile = profile;
    const constraints = captureConstraints(this.captureProfile);
    this.localStream = await navigator.mediaDevices.getUserMedia({ audio: constraints, video: false });
    const track = this.localStream.getAudioTracks()[0] ?? null;
    const settings = track?.getSettings() ?? {};
    this.captureState = {
      profile: this.captureProfile,
      requestedConstraints: constraints,
      observedSettings: {
        sampleRate: settings.sampleRate ?? null,
        channelCount: settings.channelCount ?? null,
        echoCancellation: settings.echoCancellation ?? null,
        noiseSuppression: settings.noiseSuppression ?? null,
        autoGainControl: settings.autoGainControl ?? null,
      },
    };
    if (this.audioTransceiver?.sender && track) {
      await this.audioTransceiver.sender.replaceTrack(track);
    }
    this.callbacks.onMicrophoneState(true);
    this.setPhase("microphone_ready");
  }

  async connect(): Promise<void> {
    this.assertNotStopped();
    this.setPhase("signaling_connecting");
    this.pc = new RTCPeerConnection({ iceServers: [] });
    this.resetStatsPreflight();
    this.audioTransceiver = this.pc.addTransceiver("audio", { direction: "sendrecv" });

    if (this.localStream) {
      const track = this.localStream.getAudioTracks()[0];
      if (track && this.audioTransceiver.sender) {
        await this.audioTransceiver.sender.replaceTrack(track);
      }
    }

    this.wirePeerConnection(this.pc);

    if (this.config.localPeerId === "peer_a") {
      this.dc = this.pc.createDataChannel(DATA_CHANNEL_LABEL, {
        ordered: false,
        maxRetransmits: 0,
      });
      this.setupDataChannel(this.dc, true);
    }

    const { SignalingClient } = await import("../../signaling/client");
    this.signaling = new SignalingClient({
      sessionId: this.config.sessionCorrelationId,
      peerId: this.config.localPeerId,
      url: this.config.signalingUrl,
      onMessage: (payload) => void this.handleSignaling(payload),
      onPeerJoined: (peerId) => this.handlePeerJoined(peerId),
      onPeerLeft: () => {
        this.peerPresent = false;
        this.invalidateStatsPreflight("peer left");
      },
      onError: this.callbacks.onError,
    });
    this.signaling.connect();
  }

  async startObservation(config: ObservationConfig = {}): Promise<void> {
    this.assertNotStopped();
    this.assertReadyToObserve();
    const durationSeconds = Math.min(
      config.durationSeconds ?? DEFAULT_OBSERVATION_SECONDS,
      MAX_OBSERVATION_SECONDS,
    );
    const statsIntervalMs = config.statsIntervalMs ?? DEFAULT_STATS_INTERVAL_MS;
    const probeIntervalMs = config.probeIntervalMs ?? DEFAULT_PROBE_INTERVAL_MS;
    const probeCount = Math.min(config.probeCount ?? DEFAULT_PROBE_COUNT, 120);
    this.observationStartedAt = new Date().toISOString();
    this.clockSamples = [];
    this.statsSamples = [];
    this.setPhase("observing");

    this.clockEngine?.stopProbes();
    if (this.clockEngine && this.dc?.readyState === "open") {
      this.clockEngine.start({ intervalMs: probeIntervalMs, count: probeCount });
    }
    if (this.pc) {
      this.statsSampler = new StatsSampler(this.pc, (sample) => {
        this.statsSamples.push(sample);
        this.callbacks.onStatsSample(sample);
      });
      this.statsSampler.start(
        statsIntervalMs,
        MAX_STATS_SAMPLES,
        durationSeconds * 1000,
      );
    }

    await new Promise((resolve) => setTimeout(resolve, durationSeconds * 1000));
    await this.stopObservation();
  }

  async stopObservation(): Promise<void> {
    this.clockEngine?.stopProbes();
    this.statsSampler?.stop();
    this.observationCompletedAt = new Date().toISOString();
    this.setPhase("finalizing");
    this.setPhase("completed");
  }

  exportEndpointDraft(): Record<string, unknown> {
    return this.buildEndpointExport(false);
  }

  exportFinalizedEndpoint(): Record<string, unknown> {
    if (this.phase !== "completed") {
      throw new Error("export before observation completion rejected");
    }
    const commit = this.config.softwareCommit?.trim();
    if (!commit || !LiveWebRtcSession.COMMIT_SHA40.test(commit)) {
      throw new Error("exact 40-character software commit required for finalized export");
    }
    if (!this.observationStartedAt || !this.observationCompletedAt) {
      throw new Error("observation timestamps required for finalized export");
    }
    const startMs = Date.parse(this.observationStartedAt);
    const endMs = Date.parse(this.observationCompletedAt);
    if (!Number.isFinite(startMs) || !Number.isFinite(endMs) || endMs <= startMs) {
      throw new Error("positive observation duration required for finalized export");
    }
    const durationSec = (endMs - startMs) / 1000;
    if (durationSec <= 0 || durationSec > MAX_OBSERVATION_SECONDS) {
      throw new Error("observation duration out of bounds for finalized export");
    }
    if (this.statsSamples.length < LiveWebRtcSession.MIN_FINALIZED_STATS) {
      throw new Error("insufficient stats samples for finalized export");
    }
    const localProbes = validLocalCompletedProbes(this.clockSamples, this.config.localPeerId);
    if (localProbes.length < LiveWebRtcSession.MIN_FINALIZED_PROBES) {
      throw new Error("insufficient valid local clock probes for finalized export");
    }
    return this.finalizeEndpointExport(this.buildEndpointExport(true));
  }

  private finalizeEndpointExport(exported: Record<string, unknown>): Record<string, unknown> {
    const roundTripped = JSON.parse(JSON.stringify(exported)) as Record<string, unknown>;
    const clockProbes = roundTripped.clockProbes as { samples?: import("./types").ClockProbeSample[] } | undefined;
    if (clockProbes?.samples) {
      clockProbes.samples = clockProbes.samples.map(canonicalizeClockProbeSampleForExport);
    }
    return roundTripped;
  }

  private buildEndpointExport(finalized: boolean): Record<string, unknown> {
    const medians = computeClockMedian(this.clockSamples);
    const remoteTrack = this.remoteStream?.getAudioTracks()[0] ?? null;
    const commit = this.config.softwareCommit?.trim();
    return {
      schemaVersion: LIVE_SCHEMA_VERSION,
      evidenceLevel: "browser_network_observation",
      evidenceStatus: "exploratory_non_authoritative",
      runId: `live-${this.config.sessionCorrelationId}-${this.config.localPeerId}`,
      sessionCorrelationId: this.config.sessionCorrelationId,
      peerRole: this.config.localPeerId,
      startedAtUtc: this.observationStartedAt,
      completedAtUtc: this.observationCompletedAt,
      softwareCommit: commit || null,
      exportKind: finalized ? "finalized" : "diagnostic_draft",
      environment: {
        browserFamily: detectBrowserFamily(),
        platform: navigator.platform ?? null,
        secureContext: window.isSecureContext,
        crossOriginIsolated: globalThis.crossOriginIsolated ?? false,
        locale: navigator.language,
      },
      capture: this.captureState,
      playout: {
        path: "html_media_element",
        remoteAudioTrackReceived: this.remoteStream !== null,
        remoteAudioTrackReadyState: remoteTrack?.readyState ?? null,
        remoteAudioTrackUnmuted: remoteTrack ? !remoteTrack.muted : null,
        autoplayAttempted: true,
        ...this.playoutState,
      },
      connectionLifecycle: this.connectionLifecycle,
      dataChannel: {
        label: DATA_CHANNEL_LABEL,
        ...this.dataChannelProps,
      },
      clockProbes: {
        samples: this.clockSamples.map(canonicalizeClockProbeSampleForExport),
        completedProbes: validLocalCompletedProbes(this.clockSamples, this.config.localPeerId).length,
        medianRttMs: medians.rtt,
        medianOffsetMs: medians.offset,
        madRttMs: medians.madRtt,
        offsetLimitation:
          "Clock offset is an estimate affected by route asymmetry and scheduling; not synchronized truth.",
      },
      statsSamples: this.statsSamples,
      unsupportedMetrics: {
        acousticMouthToEar: unsupported(),
        oneWayNetworkLatency: unavailable("not measured"),
      },
      limitations: [
        "Exploratory non-authoritative browser network observation.",
        "Not an acoustic mouth-to-ear measurement.",
        "No measured one-way network latency.",
        "No transport selection result.",
        "Clock offset is an estimate only.",
      ],
    };
  }

  stop(): void {
    if (this.stopped) return;
    this.stopped = true;
    this.invalidateStatsPreflight("session stopped");
    this.resetStatsPreflight();
    this.clockEngine?.stop();
    this.statsSampler?.stop();
    this.localStream?.getTracks().forEach((t) => t.stop());
    if (this.audioTransceiver?.sender) {
      void this.audioTransceiver.sender.replaceTrack(null);
    }
    this.dc?.close();
    this.pc?.close();
    this.signaling?.disconnect();
    this.iceBuffer.clear();
    this.pc = null;
    this.dc = null;
    this.signaling = null;
    this.remoteStream = null;
    this.setPhase("stopped");
  }

  getPhase(): LiveSessionPhase {
    return this.phase;
  }

  getRtpPreflightDiagnostics(): RtpPreflightDiagnostics | null {
    return this.statsPreflight?.getDiagnostics() ?? null;
  }

  private wirePeerConnection(pc: RTCPeerConnection): void {
    pc.onconnectionstatechange = () => {
      const state = pc.connectionState;
      this.connectionLifecycle.peerConnectionState = state;
      this.callbacks.onConnectionState(state);
      if (state === "connected") {
        this.setPhase("connected");
        this.evaluateReadyToObserve();
      } else if (state === "disconnected" || state === "failed" || state === "closed") {
        this.invalidateStatsPreflight(`peer connection ${state}`);
      }
    };
    pc.oniceconnectionstatechange = () => {
      this.connectionLifecycle.iceConnectionState = pc.iceConnectionState;
      this.callbacks.onIceState(pc.iceConnectionState);
      if (
        pc.iceConnectionState === "disconnected" ||
        pc.iceConnectionState === "failed" ||
        pc.iceConnectionState === "closed"
      ) {
        this.invalidateStatsPreflight(`ICE ${pc.iceConnectionState}`);
      } else {
        this.evaluateReadyToObserve();
      }
    };
    pc.onsignalingstatechange = () => {
      this.connectionLifecycle.signalingState = pc.signalingState;
      this.callbacks.onSignalingState(pc.signalingState);
    };
    pc.onnegotiationneeded = () => {
      this.callbacks.onNegotiation("negotiation_needed");
      this.maybeStartInitialNegotiation("negotiation_needed");
    };
    pc.ontrack = (event) => {
      this.remoteStream =
        event.streams[0] ??
        (event.track ? new MediaStream([event.track]) : remoteAudioStreamFromReceivers(pc));
      if (this.remoteStream) {
        this.callbacks.onRemoteStream(this.remoteStream);
        const remoteTrack = this.remoteStream.getAudioTracks()[0];
        if (remoteTrack) {
          remoteTrack.onended = () => {
            this.invalidateStatsPreflight("remote audio track ended");
          };
        }
      }
      this.evaluateReadyToObserve();
    };
    pc.onicecandidate = (event) => {
      if (event.candidate) {
        this.signaling?.relay({
          type: "ice_candidate",
          candidate: event.candidate.toJSON(),
        });
      }
    };
    pc.ondatachannel = (event) => {
      if (this.config.localPeerId === "peer_b") {
        this.setupDataChannel(event.channel, false);
      }
    };
  }

  private setupDataChannel(channel: RTCDataChannel, owner: boolean): void {
    this.dc = channel;
    this.dataChannelProps = {
      owner,
      ordered: channel.ordered,
      maxRetransmits: channel.maxRetransmits,
      negotiated: channel.negotiated,
      readyState: channel.readyState,
    };
    channel.onopen = () => {
      this.setDataChannelReadyState(channel.readyState);
      this.callbacks.onDataChannelState(channel.readyState);
      this.clockEngine = new ClockProbeEngine(this.config.localPeerId, (sample) => {
        this.clockSamples.push(sample);
        this.callbacks.onClockProbe(sample);
        this.evaluateReadyToObserve();
      });
      this.clockEngine.attach(channel);
      this.clockEngine.start({ intervalMs: 500, count: 3 });
    };
    channel.onclosing = () => {
      this.setDataChannelReadyState("closing");
      this.invalidateStatsPreflight("data channel closing");
    };
    channel.onclose = () => {
      this.setDataChannelReadyState("closed");
      this.callbacks.onDataChannelState("closed");
      this.invalidateStatsPreflight("data channel closed");
    };
    channel.onerror = () => {
      this.setDataChannelReadyState("closed");
      this.invalidateStatsPreflight("data channel error");
    };
  }

  private setDataChannelReadyState(state: RTCDataChannelState | "closing"): void {
    this.dataChannelProps.readyState = state;
  }

  private async handleSignaling(payload: unknown): Promise<void> {
    if (!this.pc) return;
    const msg = payload as {
      type?: string;
      sdp?: RTCSessionDescriptionInit;
      candidate?: RTCIceCandidateInit;
      descriptionType?: RTCSdpType;
    };

    if (msg.type === "description" || msg.type === "offer" || msg.type === "answer") {
      this.setPhase("negotiating");
      this.callbacks.onNegotiation("remote_description");
      const envelope = {
        type: "description" as const,
        sdp: msg.sdp ?? { type: msg.type as RTCSdpType, sdp: "" },
      };
      await applySignalingDescription(
        this.pc,
        this.negotiation,
        envelope,
        (type, sdp) => this.relayDescription(type, sdp),
      );
      await this.iceBuffer.flush(this.pc);
    } else if (msg.type === "ice_candidate" && msg.candidate) {
      if (this.pc.remoteDescription) {
        try {
          await this.pc.addIceCandidate(msg.candidate);
        } catch {
          this.iceBuffer.add(msg.candidate);
        }
      } else {
        this.iceBuffer.add(msg.candidate);
      }
    }
  }

  private relayDescription(type: RTCSdpType, sdp: RTCSessionDescriptionInit): void {
    this.signaling?.relay({ type: "description", descriptionType: type, sdp });
  }

  private handlePeerJoined(peerId: string): void {
    if (!peerId || peerId === this.config.localPeerId) {
      return;
    }
    this.peerPresent = true;
    this.setPhase("peer_present");
    this.maybeStartInitialNegotiation("peer_joined");
  }

  private maybeStartInitialNegotiation(
    trigger: "negotiation_needed" | "peer_joined",
  ): void {
    if (this.config.localPeerId !== "peer_a") return;
    if (!this.pc) return;
    if (!this.peerPresent) return;
    if (this.peerNegotiationStarted) return;
    if (this.negotiation.makingOffer) return;
    if (this.pc.signalingState !== "stable") return;
    if (this.stopped) return;

    this.peerNegotiationStarted = true;
    void this.startNegotiation().catch((error) => {
      if (
        !this.stopped &&
        this.pc?.signalingState === "stable" &&
        !this.negotiation.makingOffer
      ) {
        this.peerNegotiationStarted = false;
        if (this.peerPresent) {
          this.setPhase("peer_present");
        }
      }
      const message = error instanceof Error ? error.message : String(error);
      this.callbacks.onError(`initial negotiation failed (${trigger}): ${message}`);
    });
  }

  private async startNegotiation(): Promise<void> {
    if (!this.pc) return;
    this.setPhase("negotiating");
    this.callbacks.onNegotiation("local_offer");
    await createAndSendOffer(this.pc, this.negotiation, (type, sdp) =>
      this.relayDescription(type, sdp),
    );
  }

  private evaluateReadyToObserve(): void {
    if (this.stopped) return;
    this.ensureStatsPreflight();
    if (!this.remoteStream && this.pc) {
      const receiverStream = remoteAudioStreamFromReceivers(this.pc);
      if (receiverStream) {
        this.remoteStream = receiverStream;
        this.callbacks.onRemoteStream(receiverStream);
      }
    }
    const failures = this.collectReadyFailures();
    if (
      failures.length === 0 &&
      this.phase !== "observing" &&
      this.phase !== "completed" &&
      this.phase !== "finalizing" &&
      this.phase !== "ready_to_observe"
    ) {
      this.setPhase("ready_to_observe");
    }
  }

  private canStartStatsPreflight(): boolean {
    if (this.stopped || !this.pc || !this.peerPresent) return false;
    if (this.pc.connectionState !== "connected") return false;
    const ice = this.pc.iceConnectionState;
    if (ice !== "connected" && ice !== "completed") return false;
    if (this.dc?.readyState !== "open") return false;
    const remoteTrack = this.remoteStream?.getAudioTracks()[0];
    if (!remoteTrack || remoteTrack.readyState !== "live") return false;
    return true;
  }

  private resetStatsPreflight(): void {
    this.statsPreflight?.reset();
    this.statsPreflight = null;
  }

  private invalidateStatsPreflight(reason: string): void {
    this.statsPreflight?.invalidate(reason);
    this.evaluateReadyToObserve();
  }

  private ensureStatsPreflight(): void {
    if (!this.canStartStatsPreflight()) {
      const state = this.statsPreflight?.getState();
      if (state === "probing" || state === "available" || state === "exhausted") {
        this.invalidateStatsPreflight("preflight prerequisites lost");
      }
      return;
    }
    if (!this.statsPreflight && this.pc) {
      this.statsPreflight = new RtpStatsPreflightController(
        this.pc,
        DEFAULT_RTP_PREFLIGHT_CONFIG,
        {
          onUpdate: () => this.evaluateReadyToObserve(),
        },
      );
    }
    const state = this.statsPreflight?.getState();
    if (state === "cancelled" || state === "idle") {
      this.statsPreflight?.restartWhenIdle();
    } else if (state !== "available" && state !== "probing" && state !== "exhausted") {
      this.statsPreflight?.maybeStart();
    }
  }

  private assertReadyToObserve(): void {
    const failures = this.collectReadyFailures();
    if (failures.length > 0) {
      throw new Error(`not ready to observe: ${failures.join(", ")}`);
    }
  }

  private collectReadyFailures(): string[] {
    const failures: string[] = [];
    if (!this.peerPresent) failures.push("peer not present");
    const micTrack = this.localStream?.getAudioTracks()[0];
    if (!micTrack || micTrack.readyState !== "live") failures.push("microphone track not live");
    if (this.pc?.connectionState !== "connected") failures.push("peer connection not connected");
    const ice = this.pc?.iceConnectionState;
    if (ice !== "connected" && ice !== "completed") failures.push("ICE not connected");
    if (this.pc?.signalingState !== "stable") failures.push("signaling not stable");
    if (this.negotiation.makingOffer || this.negotiation.isSettingRemoteAnswerPending) {
      failures.push("negotiation in progress");
    }
    const remoteTrack = this.remoteStream?.getAudioTracks()[0];
    if (!remoteTrack || remoteTrack.readyState === "ended") {
      failures.push("remote audio track missing or ended");
    }
    if (this.dc?.readyState !== "open") failures.push("data channel not open");
    const localProbes = validLocalCompletedProbes(this.clockSamples, this.config.localPeerId);
    if (localProbes.length < 1) failures.push("clock preflight incomplete");
    const preflightState = this.statsPreflight?.getState() ?? "idle";
    if (preflightState !== "available") {
      if (preflightState === "exhausted") {
        failures.push("genuine RTP audio stats unavailable");
      } else {
        failures.push("stats preflight incomplete");
      }
    }
    const commit = this.config.softwareCommit?.trim();
    if (!commit || !LiveWebRtcSession.COMMIT_SHA40.test(commit)) {
      failures.push("exact software commit missing");
    }
    return failures;
  }

  private setPhase(phase: LiveSessionPhase): void {
    this.phase = phase;
    this.callbacks.onPhaseChange(phase);
  }

  private assertNotStopped(): void {
    if (this.stopped) throw new Error("session stopped");
  }
}

function detectBrowserFamily(): string | null {
  const ua = navigator.userAgent;
  if (ua.includes("Chrome")) return "chromium";
  if (ua.includes("Firefox")) return "firefox";
  if (ua.includes("Safari")) return "safari";
  return null;
}

function remoteAudioStreamFromReceivers(pc: RTCPeerConnection): MediaStream | null {
  const tracks = pc
    .getReceivers()
    .map((receiver) => receiver.track)
    .filter((track): track is MediaStreamTrack => track?.kind === "audio");
  return tracks.length > 0 ? new MediaStream(tracks) : null;
}

export type { RtpPreflightDiagnostics };
export type { LiveSessionConfig, LiveSessionCallbacks, PeerRole, CaptureProfile, ObservationConfig };
