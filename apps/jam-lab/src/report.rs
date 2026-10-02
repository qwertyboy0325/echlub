//! Markdown renderings for lab outputs.

use std::fmt::Write;

use echlub_jam::budget::PathBudget;
use echlub_jam::pipeline::LatencyStats;
use echlub_musician_sim::ensemble::SweepRow;

use crate::bots::BandResult;
use crate::scenario::ScenarioReport;

pub const SIM_BANNER: &str = "> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.";

pub fn scenario_markdown(r: &ScenarioReport) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# Jam scenario: {}\n\n{SIM_BANNER}\n", r.name);
    let _ = writeln!(
        s,
        "- Endpoint: `{}`  \n- Topology: {:?}  \n- Cross-ISP penalty: {}  \n- Tempo: {} BPM, {} bars  \n- Ensemble runs: {}\n",
        r.endpoint.name, r.pipeline.topology, r.cross_isp, r.bpm, r.bars, r.ensemble_runs
    );
    let _ = writeln!(s, "## Players\n\n| # | Player | Jitter depth (frames) | Link expected one-way (ms) |\n| --- | --- | --- | --- |");
    for (i, p) in r.pipeline.peers.iter().enumerate() {
        let _ = writeln!(
            s,
            "| {i} | {} | {} | {:.1} |",
            p.name,
            p.jitter_depth,
            p.link.expected_ms()
        );
    }
    let _ = writeln!(s, "\n## Mouth-to-ear latency (pipeline simulation + analytic device budget)\n\n| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected |\n| --- | --- | --- | --- | --- |");
    for p in &r.pipeline.pairs {
        let _ = writeln!(
            s,
            "| {} → {} | {} | {} | {:.1} | {}/{} |",
            p.from,
            p.to,
            fmt_opt(p.mouth_to_ear_p50_ms),
            fmt_opt(p.mouth_to_ear_p95_ms),
            p.analytic_budget_ms,
            p.detected_probes,
            p.expected_probes
        );
    }
    let _ = writeln!(
        s,
        "\n## Simulated ensemble\n\n| Metric | Value |\n| --- | --- |"
    );
    let _ = writeln!(s, "| Tight fraction | {:.0}% |", r.tight_fraction * 100.0);
    let _ = writeln!(
        s,
        "| Playable-or-better fraction | {:.0}% |",
        r.playable_or_better_fraction * 100.0
    );
    let _ = writeln!(s, "| Mean tempo drift | {:+.2}% |", r.mean_tempo_drift_pct);
    let _ = writeln!(
        s,
        "| Mean worst heard RMS asynchrony | {:.1} ms |",
        r.mean_worst_heard_rms_ms
    );
    let t = r.thresholds;
    let _ = writeln!(
        s,
        "\nVerdict thresholds (hypotheses): Tight = |drift| ≤ {}% and heard RMS ≤ {} ms; Playable = |drift| ≤ {}% and heard RMS ≤ {} ms.",
        t.tight_drift_pct, t.tight_heard_rms_ms, t.playable_drift_pct, t.playable_heard_rms_ms
    );
    s
}

pub fn budget_markdown(budgets: &[PathBudget]) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# Latency budget (analytic)\n\n{SIM_BANNER}\n");
    for b in budgets {
        let _ = writeln!(
            s,
            "## {} → {}: {:.1} ms\n\n| Component | ms |\n| --- | --- |",
            b.from, b.to, b.total_ms
        );
        for i in &b.items {
            if i.ms > 0.0 {
                let _ = writeln!(s, "| {} | {:.2} |", i.label, i.ms);
            }
        }
        let _ = writeln!(s);
    }
    s
}

pub fn sweep_markdown(rows: &[SweepRow], bpm: f64, compensation: f64) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "# Latency tolerance sweep: rock four-piece, {bpm} BPM, compensation {compensation}\n\n{SIM_BANNER}\n\n| Uniform one-way latency (ms) | Mean tempo drift | Mean worst heard RMS (ms) | Tight | Playable+ |\n| --- | --- | --- | --- | --- |"
    );
    for r in rows {
        let _ = writeln!(
            s,
            "| {:.0} | {:+.2}% | {:.1} | {:.0}% | {:.0}% |",
            r.latency_ms,
            r.mean_tempo_drift_pct,
            r.mean_worst_heard_rms_ms,
            r.tight_fraction * 100.0,
            r.playable_or_better_fraction * 100.0
        );
    }
    s
}

pub fn band_markdown(r: &BandResult, peers: usize, label: &str) -> String {
    let mut s = String::new();
    let _ = writeln!(
        s,
        "# Bot band over UDP: {label}\n\n> Real sockets, synthetic musicians, impairment injected in-process. Localhost results describe this machine only.\n\n| Bot | Sent | Uplink drops | Mixes recv | Downlink drops | Concealed | Late ticks |\n| --- | --- | --- | --- | --- | --- | --- |"
    );
    for b in &r.bots {
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} |",
            b.peer_id,
            b.frames_sent,
            b.frames_dropped_uplink,
            b.mixes_received,
            b.mixes_dropped_downlink,
            b.jitter.concealed,
            b.late_ticks
        );
    }
    let any_probe = r.bots.iter().any(|b| !b.detections.is_empty());
    if any_probe {
        let _ = writeln!(s, "\n## Probe latency (capture → playout, no device buffers)\n\n| From → To | n | p50 (ms) | p95 (ms) | max (ms) |\n| --- | --- | --- | --- | --- |");
        for b in &r.bots {
            for src in (0..peers).filter(|p| *p != b.peer_id as usize) {
                let v: Vec<f64> = b
                    .detections
                    .iter()
                    .filter(|(e, _)| *e == src)
                    .map(|(_, l)| *l)
                    .collect();
                match LatencyStats::from_samples(&v) {
                    Some(st) => {
                        let _ = writeln!(
                            s,
                            "| {src} → {} | {} | {:.1} | {:.1} | {:.1} |",
                            b.peer_id, st.count, st.p50_ms, st.p95_ms, st.max_ms
                        );
                    }
                    None => {
                        let _ = writeln!(s, "| {src} → {} | 0 | — | — | — |", b.peer_id);
                    }
                }
            }
        }
    }
    if let Some(relay) = &r.relay {
        let _ = writeln!(
            s,
            "\nRelay: {} ticks, {} late, {} packets in, {} out, {} decode errors.",
            relay.ticks, relay.late_ticks, relay.packets_in, relay.packets_out, relay.decode_errors
        );
    }
    s
}

fn fmt_opt(v: Option<f64>) -> String {
    v.map(|x| format!("{x:.1}")).unwrap_or_else(|| "—".into())
}
