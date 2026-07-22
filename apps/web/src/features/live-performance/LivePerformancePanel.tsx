import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { DEFAULT_SIGNALING_URL } from "../../adapters/signaling/client";
import { LiveWebRtcSession } from "../../adapters/webrtc/live/session";
import {
  generateSessionCorrelationId,
  type CaptureProfile,
  type LiveSessionPhase,
  type PeerRole,
} from "../../adapters/webrtc/live/types";
import { downloadJson } from "../../shared/performance/types";
import { injectedSoftwareCommit } from "../../shared/build-commit";

const PRIVACY_NOTICE =
  "Use headphones. Raw microphone audio is exchanged through WebRTC and is not stored by EchLub. " +
  "The signaling server relays negotiation messages only. Exported evidence excludes SDP, ICE addresses, " +
  "device identifiers, device labels, and raw audio. This is not an acoustic mouth-to-ear measurement.";

const PHASE_LABELS: Record<LiveSessionPhase, string> = {
  idle: "Idle",
  prepared: "Prepared",
  microphone_ready: "Microphone Ready",
  signaling_connecting: "Signaling Connecting",
  peer_present: "Peer Present",
  negotiating: "Negotiating",
  connected: "Connected",
  ready_to_observe: "Ready To Observe",
  observing: "Observing",
  finalizing: "Finalizing",
  completed: "Completed",
  failed: "Failed",
  stopped: "Stopped",
};

export function LivePerformancePanel() {
  const [role, setRole] = useState<PeerRole>("peer_a");
  const [correlationId, setCorrelationId] = useState(generateSessionCorrelationId);
  const [signalingUrl, setSignalingUrl] = useState(DEFAULT_SIGNALING_URL);
  const [captureProfile, setCaptureProfile] = useState<CaptureProfile>("browser_default");
  const [phase, setPhase] = useState<LiveSessionPhase>("idle");
  const [connectionState, setConnectionState] = useState("new");
  const [iceState, setIceState] = useState("new");
  const [dcState, setDcState] = useState("closed");
  const [sampleCount, setSampleCount] = useState(0);
  const [probeCount, setProbeCount] = useState(0);
  const [clockRttMedian, setClockRttMedian] = useState<number | null>(null);
  const [elapsed, setElapsed] = useState(0);
  const [headphonesAck, setHeadphonesAck] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const sessionRef = useRef<LiveWebRtcSession | null>(null);
  const probeValues = useRef<number[]>([]);
  const observeTimer = useRef<ReturnType<typeof setInterval> | null>(null);

  const canPrepare = phase === "idle";
  const canEnableMic = headphonesAck && (phase === "prepared" || phase === "idle");
  const canConnect = phase === "microphone_ready";
  const canObserve = phase === "ready_to_observe";
  const canExport = phase === "completed";
  const canStop = phase !== "idle" && phase !== "stopped";

  useEffect(() => () => {
    sessionRef.current?.stop();
    if (observeTimer.current) clearInterval(observeTimer.current);
  }, []);

  const callbacks = useMemo(
    () => ({
      onPhaseChange: setPhase,
      onSignalingState: () => {},
      onConnectionState: setConnectionState,
      onIceState: setIceState,
      onDataChannelState: setDcState,
      onRemoteStream: (stream: MediaStream) => {
        const audio = document.getElementById("remote-audio") as HTMLAudioElement | null;
        if (audio) audio.srcObject = stream;
      },
      onMicrophoneState: () => {},
      onNegotiation: () => {},
      onClockProbe: (sample: { rttMs: number | null; timeout: boolean }) => {
        if (sample.rttMs !== null && !sample.timeout) {
          probeValues.current.push(sample.rttMs);
          const sorted = [...probeValues.current].sort((a, b) => a - b);
          setClockRttMedian(sorted[Math.floor(sorted.length / 2)] ?? null);
        }
        setProbeCount((c) => c + 1);
      },
      onStatsSample: () => setSampleCount((c) => c + 1),
      onError: setError,
    }),
    [],
  );

  const prepare = useCallback(() => {
    setError(null);
    const session = new LiveWebRtcSession(
      {
        sessionCorrelationId: correlationId,
        localPeerId: role,
        signalingUrl,
        captureProfile,
        softwareCommit: injectedSoftwareCommit(),
      },
      callbacks,
    );
    session.prepare();
    sessionRef.current = session;
  }, [callbacks, captureProfile, correlationId, role, signalingUrl]);

  const enableMic = useCallback(async () => {
    await sessionRef.current?.enableMicrophone(captureProfile);
  }, [captureProfile]);

  const connect = useCallback(async () => {
    await sessionRef.current?.connect();
  }, []);

  const startObservation = useCallback(async () => {
    setElapsed(0);
    observeTimer.current = setInterval(() => setElapsed((e) => e + 1), 1000);
    await sessionRef.current?.startObservation({ durationSeconds: 60, probeCount: 30 });
    if (observeTimer.current) clearInterval(observeTimer.current);
  }, []);

  const exportFinalized = useCallback(() => {
    try {
      const endpoint = sessionRef.current?.exportFinalizedEndpoint();
      if (!endpoint) return;
      if (endpoint.exportKind !== "finalized") {
        throw new Error("finalized export produced non-finalized artifact");
      }
      downloadJson(`${role}-${correlationId.slice(0, 8)}-finalized.json`, endpoint);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [correlationId, role]);

  const exportDiagnosticDraft = useCallback(() => {
    try {
      const draft = sessionRef.current?.exportEndpointDraft();
      if (!draft) return;
      downloadJson(`${role}-${correlationId.slice(0, 8)}-diagnostic_draft.json`, draft);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, [correlationId, role]);

  const disconnect = useCallback(() => {
    sessionRef.current?.stop();
    sessionRef.current = null;
    setPhase("stopped");
  }, []);

  const reset = useCallback(() => {
    disconnect();
    setCorrelationId(generateSessionCorrelationId());
    setPhase("idle");
    setSampleCount(0);
    setProbeCount(0);
    setClockRttMedian(null);
    setElapsed(0);
    setHeadphonesAck(false);
    probeValues.current = [];
  }, [disconnect]);

  return (
    <section className="performance-panel">
      <h2>Live Two-Peer Observation</h2>
      <p className="privacy-notice">{PRIVACY_NOTICE}</p>

      <div className="controls">
        <label>
          Session correlation ID
          <input value={correlationId} readOnly />
          <button type="button" onClick={() => setCorrelationId(generateSessionCorrelationId())}>
            Generate
          </button>
          <button type="button" onClick={() => navigator.clipboard.writeText(correlationId)}>
            Copy
          </button>
        </label>
        <label>
          Signaling URL (not exported)
          <input value={signalingUrl} onChange={(e) => setSignalingUrl(e.target.value)} />
        </label>
        <label>
          Role
          <select value={role} onChange={(e) => setRole(e.target.value as PeerRole)}>
            <option value="peer_a">Peer A</option>
            <option value="peer_b">Peer B</option>
          </select>
        </label>
        <label>
          Capture profile
          <select value={captureProfile} onChange={(e) => setCaptureProfile(e.target.value as CaptureProfile)}>
            <option value="browser_default">browser_default</option>
            <option value="music_low_latency">music_low_latency</option>
          </select>
        </label>
        <label>
          <input type="checkbox" checked={headphonesAck} onChange={(e) => setHeadphonesAck(e.target.checked)} />
          I am using headphones and understand this is an experimental live-audio session.
        </label>
      </div>

      <div className="controls">
        <button onClick={prepare} disabled={!canPrepare}>Prepare</button>
        <button onClick={enableMic} disabled={!canEnableMic}>Enable Microphone</button>
        <button onClick={connect} disabled={!canConnect}>Connect</button>
        <button onClick={startObservation} disabled={!canObserve}>Start 60s Observation</button>
        <button onClick={exportFinalized} disabled={!canExport}>Export Finalized Endpoint</button>
        <button onClick={exportDiagnosticDraft} disabled={phase === "idle" || phase === "stopped"}>
          Export Diagnostic Draft
        </button>
        <button onClick={disconnect} disabled={!canStop}>Disconnect</button>
        <button onClick={reset}>Reset</button>
      </div>

      <p>Phase: {PHASE_LABELS[phase]}</p>
      <p>Connection: {connectionState} | ICE: {iceState} | DataChannel: {dcState}</p>
      <p>Samples: {sampleCount} | Probes: {probeCount} | Elapsed: {elapsed}s</p>
      {clockRttMedian !== null && (
        <p>Clock-probe RTT median: {clockRttMedian.toFixed(2)} ms (estimate; not one-way latency)</p>
      )}
      {error && <p className="error">{error}</p>}
      <audio id="remote-audio" autoPlay />
    </section>
  );
}
