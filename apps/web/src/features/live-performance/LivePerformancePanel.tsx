import { useCallback, useRef, useState } from "react";
import { LiveWebRtcSession } from "../../adapters/webrtc/live/session";
import {
  downloadJson,
  EVIDENCE_STATUS,
  PERFORMANCE_DISCLAIMER,
  SCHEMA_VERSION,
  sanitizeForExport,
  unavailable,
  unsupported,
  type PerformanceRunDraft,
} from "../../shared/performance/types";

type PeerRole = "peer-a" | "peer-b";

export function LivePerformancePanel() {
  const [role, setRole] = useState<PeerRole>("peer-a");
  const [sessionId, setSessionId] = useState("live-session-001");
  const [connectionState, setConnectionState] = useState<string>("new");
  const [micEnabled, setMicEnabled] = useState(false);
  const [clockRtt, setClockRtt] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const sessionRef = useRef<LiveWebRtcSession | null>(null);

  const startSession = useCallback(async () => {
    setError(null);
    const localPeerId = role;
    const remotePeerId = role === "peer-a" ? "peer-b" : "peer-a";

    const session = new LiveWebRtcSession(
      { sessionId, localPeerId, remotePeerId },
      {
        onConnectionState: setConnectionState,
        onRemoteStream: (stream) => {
          const audio = document.getElementById("remote-audio") as HTMLAudioElement | null;
          if (audio) audio.srcObject = stream;
        },
        onDataChannelMessage: () => {},
        onClockProbe: setClockRtt,
        onError: setError,
      },
    );
    sessionRef.current = session;
    await session.start();
  }, [role, sessionId]);

  const enableMic = useCallback(async () => {
    await sessionRef.current?.enableMicrophone();
    setMicEnabled(true);
  }, []);

  const probeClock = useCallback(() => {
    sessionRef.current?.sendClockProbe();
  }, []);

  const exportEvidence = useCallback(() => {
    const draft: PerformanceRunDraft = {
      schemaVersion: SCHEMA_VERSION,
      evidenceLevel: "browser_network_observation",
      evidenceStatus: EVIDENCE_STATUS,
      metadata: {
        runId: `live-${sessionId}-${Date.now()}`,
        startedAt: new Date().toISOString(),
        completedAt: new Date().toISOString(),
        harnessVersion: "0.1.0",
        browserFamily: null,
        platform: navigator.platform ?? null,
      },
      timing: {
        pulseEmitMs: unsupported(),
        pulseDetectMs: unsupported(),
        loopbackLatencyMs: unsupported(),
        datachannelRttMs: clockRtt !== null ? { kind: "observed", value: clockRtt } : unavailable("no probe sent"),
        iceGatheringMs: unsupported(),
        connectionSetupMs: unsupported(),
      },
      syntheticPulse: {
        pulsesEmitted: unsupported(),
        pulsesDetected: unsupported(),
        detectionRate: unsupported(),
        meanDetectionLatencyMs: unsupported(),
        jitterMs: unsupported(),
      },
      transport: {
        candidatePairType: unsupported(),
        bytesSent: unsupported(),
        bytesReceived: unsupported(),
        packetsLost: unsupported(),
      },
    };
    downloadJson(`live-performance-${sessionId}.json`, sanitizeForExport(draft));
  }, [sessionId, clockRtt]);

  const stopSession = useCallback(() => {
    sessionRef.current?.stop();
    sessionRef.current = null;
    setMicEnabled(false);
    setConnectionState("closed");
  }, []);

  return (
    <section className="performance-panel">
      <h2>Live Performance Session</h2>
      <p className="disclaimer">{PERFORMANCE_DISCLAIMER}</p>
      <p className="privacy-notice">
        Privacy: no IP addresses, SDP, ICE candidates, or device identifiers are stored in exported evidence.
        Microphone access requires explicit button click.
      </p>

      <div className="controls">
        <label>
          Session ID
          <input value={sessionId} onChange={(e) => setSessionId(e.target.value)} />
        </label>
        <label>
          Role
          <select value={role} onChange={(e) => setRole(e.target.value as PeerRole)}>
            <option value="peer-a">Peer A</option>
            <option value="peer-b">Peer B</option>
          </select>
        </label>
        <button onClick={startSession}>Connect</button>
        <button onClick={enableMic} disabled={!sessionRef.current || micEnabled}>
          Enable Microphone
        </button>
        <button onClick={probeClock} disabled={connectionState !== "connected"}>
          Clock Probe
        </button>
        <button onClick={exportEvidence}>Export Local JSON</button>
        <button onClick={stopSession}>Disconnect</button>
      </div>

      <p>Connection: {connectionState}</p>
      {clockRtt !== null && <p>DataChannel RTT: {clockRtt.toFixed(2)} ms</p>}
      {error && <p className="error">{error}</p>}
      <audio id="remote-audio" autoPlay />
    </section>
  );
}
