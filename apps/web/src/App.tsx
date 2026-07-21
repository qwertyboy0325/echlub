import { useEffect, useState } from "react";
import { EchlubCoreFacade, loadEchlubCore } from "./wasm/loader";
import "./App.css";

type LogEntry = { action: string; decision: string };

export default function App() {
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
    if (!core) {
      return;
    }
    const result = fn(core);
    setLog((entries) => [{ action, decision: result }, ...entries].slice(0, 8));
    refresh(core);
  }

  return (
    <main className="app">
      <header className="banner">
        Foundation laboratory — no live audio implementation yet
      </header>

      <p className="note">
        Edit intent and network transport are not connected in this foundation package.
      </p>

      {error && <p className="error">WASM unavailable: {error}</p>}

      <section className="actions">
        <button disabled={!core} onClick={() => run("add_track", (c) => c.add_track(1n, "Lab Track"))}>
          Create track
        </button>
        <button
          disabled={!core}
          onClick={() => run("add_note", (c) => c.add_note(1n, 1n, 0n, 480n, 60, 100))}
        >
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
    </main>
  );
}
