import { useEffect, useState } from "react";
import { EchlubCoreFacade, loadEchlubCore } from "./wasm/loader";
import { LivePerformancePanel } from "./features/live-performance/LivePerformancePanel";
import { SyntheticPerformancePanel } from "./features/synthetic-performance/SyntheticPerformancePanel";
import "./App.css";

type LogEntry = { action: string; decision: string };
type Section = "foundation" | "synthetic" | "live";

function sectionFromQuery(): Section {
  const params = new URLSearchParams(window.location.search);
  const section = params.get("section");
  if (section === "synthetic" || section === "live") return section;
  return "foundation";
}

export default function App() {
  const [section, setSection] = useState<Section>(sectionFromQuery);
  const [core, setCore] = useState<EchlubCoreFacade | null>(null);
  const [snapshot, setSnapshot] = useState("{}");
  const [hash, setHash] = useState("");
  const [decision, setDecision] = useState("");
  const [log, setLog] = useState<LogEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadEchlubCore()
      .then((Core) => {
        const instance = new Core(1n, 1n);
        setCore(instance);
        refresh(instance);
      })
      .catch((err: Error) => setError(err.message));
  }, []);

  function refresh(instance: EchlubCoreFacade) {
    setSnapshot(instance.snapshot_json());
    setHash(instance.state_hash_hex());
    setDecision(instance.last_decision_json());
  }

  function run(action: string, fn: (instance: EchlubCoreFacade) => string) {
    if (!core) return;
    const result = fn(core);
    setLog((entries) => [{ action, decision: result }, ...entries].slice(0, 8));
    refresh(core);
  }

  return (
    <main className="app">
      <header className="banner">
        EchLub Laboratory — foundation + performance baseline
      </header>

      <nav className="section-nav">
        <button className={section === "foundation" ? "active" : ""} onClick={() => setSection("foundation")}>
          Foundation
        </button>
        <button className={section === "synthetic" ? "active" : ""} onClick={() => setSection("synthetic")}>
          Synthetic Loopback
        </button>
        <button className={section === "live" ? "active" : ""} onClick={() => setSection("live")}>
          Live Session
        </button>
      </nav>

      {section === "foundation" && (
        <>
          <p className="note">Composition foundation — domain mutations via WASM only.</p>
          {error && <p className="error">WASM unavailable: {error}</p>}
          <section className="actions">
            <button disabled={!core} onClick={() => run("add_track", (c) => c.add_track(1n, "Lab Track"))}>
              Create track
            </button>
            <button disabled={!core} onClick={() => run("add_note", (c) => c.add_note(1n, 1n, 0n, 480n, 60, 100))}>
              Add note
            </button>
            <button disabled={!core} onClick={() => run("move_note", (c) => c.move_note(1n, 1n, 960n))}>
              Move note
            </button>
            <button disabled={!core} onClick={() => run("delete_note", (c) => c.delete_note(1n, 2n))}>
              Delete note
            </button>
          </section>
          <section>
            <h2>State hash</h2>
            <code>{hash || "loading..."}</code>
          </section>
          <section>
            <h2>Latest decision</h2>
            <code>{decision || "loading..."}</code>
          </section>
          <section>
            <h2>Snapshot JSON</h2>
            <pre>{snapshot}</pre>
          </section>
          <section>
            <h2>Operation log</h2>
            <ul>
              {log.map((entry, index) => (
                <li key={`${entry.action}-${index}`}>
                  {entry.action}: {entry.decision}
                </li>
              ))}
            </ul>
          </section>
        </>
      )}

      {section === "synthetic" && <SyntheticPerformancePanel />}
      {section === "live" && <LivePerformancePanel />}
    </main>
  );
}
