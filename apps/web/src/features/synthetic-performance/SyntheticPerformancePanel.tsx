import { useCallback, useState } from "react";
import {
  buildPerformanceRunDraft,
  runSyntheticLoopback,
} from "../../adapters/webrtc/synthetic/loopback";
import {
  downloadJson,
  PERFORMANCE_DISCLAIMER,
  sanitizeForExport,
} from "../../shared/performance/types";

export function SyntheticPerformancePanel() {
  const [running, setRunning] = useState(false);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const runSynthetic = useCallback(async () => {
    setRunning(true);
    setError(null);
    setResult(null);
    try {
      const data = await runSyntheticLoopback();
      setResult(JSON.stringify(data, null, 2));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRunning(false);
    }
  }, []);

  const exportEvidence = useCallback(async () => {
    try {
      const data = await runSyntheticLoopback();
      const draft = buildPerformanceRunDraft(data);
      downloadJson(`synthetic-performance-${Date.now()}.json`, sanitizeForExport(draft));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    }
  }, []);

  return (
    <section className="performance-panel">
      <h2>Synthetic WebRTC Loopback</h2>
      <p className="disclaimer">{PERFORMANCE_DISCLAIMER}</p>
      <p>
        Path: AudioContext → pulse → MediaStreamDestination → RTCPeerConnection A/B → AudioWorklet detector
      </p>

      <div className="controls">
        <button onClick={runSynthetic} disabled={running}>
          {running ? "Running…" : "Run Synthetic Loopback"}
        </button>
        <button onClick={exportEvidence} disabled={running}>
          Export Evidence JSON
        </button>
      </div>

      {error && <p className="error">{error}</p>}
      {result && (
        <pre className="result-preview">{result}</pre>
      )}
    </section>
  );
}
