use crate::schema::{MetricValue, PerformanceRunV1};

pub const REPORT_DISCLAIMER: &str = "Exploratory non-authoritative performance observation. Not an acoustic mouth-to-ear measurement. Not a transport selection result.";

#[derive(Debug, PartialEq, Eq)]
pub struct SummaryReport {
    pub markdown: String,
}

fn format_metric(name: &str, metric: &MetricValue) -> String {
    match metric {
        MetricValue::Observed { value } => format!("| {name} | {value:.3} | observed |"),
        MetricValue::Unsupported => format!("| {name} | — | unsupported |"),
        MetricValue::Unavailable { reason } => format!("| {name} | — | unavailable: {reason} |"),
        MetricValue::Invalid { reason } => format!("| {name} | — | invalid: {reason} |"),
    }
}

pub fn summarize_run(run: &PerformanceRunV1) -> SummaryReport {
    let mut lines = vec![
        "# Performance Run Summary".to_string(),
        String::new(),
        format!("> {REPORT_DISCLAIMER}"),
        String::new(),
        format!("- **Run ID**: {}", run.metadata.run_id),
        format!("- **Evidence level**: {:?}", run.evidence_level),
        format!("- **Evidence status**: {:?}", run.evidence_status),
        format!("- **Started**: {}", run.metadata.started_at),
        format!("- **Completed**: {}", run.metadata.completed_at),
        String::new(),
        "## Timing Metrics".to_string(),
        String::new(),
        "| Metric | Value | Status |".to_string(),
        "| --- | --- | --- |".to_string(),
        format_metric("Pulse emit (ms)", &run.timing.pulse_emit_ms),
        format_metric("Pulse detect (ms)", &run.timing.pulse_detect_ms),
        format_metric("Loopback latency (ms)", &run.timing.loopback_latency_ms),
        format_metric("DataChannel RTT (ms)", &run.timing.datachannel_rtt_ms),
        format_metric("ICE gathering (ms)", &run.timing.ice_gathering_ms),
        format_metric("Connection setup (ms)", &run.timing.connection_setup_ms),
        String::new(),
        "## Synthetic Pulse Metrics".to_string(),
        String::new(),
        "| Metric | Value | Status |".to_string(),
        "| --- | --- | --- |".to_string(),
        format_metric("Pulses emitted", &run.synthetic_pulse.pulses_emitted),
        format_metric("Pulses detected", &run.synthetic_pulse.pulses_detected),
        format_metric("Detection rate", &run.synthetic_pulse.detection_rate),
        format_metric(
            "Mean detection latency (ms)",
            &run.synthetic_pulse.mean_detection_latency_ms,
        ),
        format_metric("Jitter (ms)", &run.synthetic_pulse.jitter_ms),
        String::new(),
        "## Transport Observation".to_string(),
        String::new(),
        "| Metric | Value | Status |".to_string(),
        "| --- | --- | --- |".to_string(),
        format_metric("Candidate pair type", &run.transport.candidate_pair_type),
        format_metric("Bytes sent", &run.transport.bytes_sent),
        format_metric("Bytes received", &run.transport.bytes_received),
        format_metric("Packets lost", &run.transport.packets_lost),
    ];

    if let Some(derived) = &run.derived {
        lines.push(String::new());
        lines.push("## Derived Metrics (Rust canonical)".to_string());
        lines.push(String::new());
        lines.push("| Metric | Value | Status |".to_string());
        lines.push("| --- | --- | --- |".to_string());
        lines.push(format_metric(
            "End-to-end synthetic (ms)",
            &derived.end_to_end_synthetic_ms,
        ));
        lines.push(format_metric(
            "Setup to first pulse (ms)",
            &derived.setup_to_first_pulse_ms,
        ));
        lines.push(format_metric(
            "DataChannel overhead (ms)",
            &derived.datachannel_overhead_ms,
        ));
    }

    SummaryReport {
        markdown: lines.join("\n"),
    }
}
