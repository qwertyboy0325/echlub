use crate::schema::{DerivedMetrics, MetricValue, PerformanceRunV1};

fn observed(value: f64) -> MetricValue {
    MetricValue::Observed { value }
}

fn unavailable(reason: &str) -> MetricValue {
    MetricValue::Unavailable {
        reason: reason.to_string(),
    }
}

fn extract_observed(metric: &MetricValue) -> Option<f64> {
    match metric {
        MetricValue::Observed { value } => Some(*value),
        _ => None,
    }
}

pub fn compute_derived_metrics(run: &PerformanceRunV1) -> DerivedMetrics {
    let loopback = extract_observed(&run.timing.loopback_latency_ms);
    let setup = extract_observed(&run.timing.connection_setup_ms);
    let rtt = extract_observed(&run.timing.datachannel_rtt_ms);
    let pulses_detected = extract_observed(&run.synthetic_pulse.pulses_detected);

    if pulses_detected.map(|v| v <= 0.0).unwrap_or(true) {
        return DerivedMetrics {
            end_to_end_synthetic_ms: unavailable(
                "no pulse detections for end-to-end synthetic metric",
            ),
            setup_to_first_pulse_ms: setup
                .map(observed)
                .unwrap_or_else(|| unavailable("connection setup timing unavailable")),
            datachannel_overhead_ms: unavailable("loopback latency unavailable without detections"),
        };
    }

    let end_to_end_synthetic_ms = match (setup, loopback) {
        (Some(s), Some(l)) => observed(s + l),
        (None, Some(l)) => observed(l),
        _ => unavailable("insufficient timing observations for end-to-end synthetic metric"),
    };

    let setup_to_first_pulse_ms = setup
        .map(observed)
        .unwrap_or_else(|| unavailable("connection setup timing unavailable"));

    let datachannel_overhead_ms = match (loopback, rtt) {
        (Some(l), Some(r)) if l >= r => observed(l - r),
        (Some(l), Some(r)) if l < r => MetricValue::Invalid {
            reason: format!("loopback ({l}) less than datachannel RTT ({r})"),
        },
        (Some(l), None) => observed(l),
        _ => unavailable("loopback latency unavailable for datachannel overhead"),
    };

    DerivedMetrics {
        end_to_end_synthetic_ms,
        setup_to_first_pulse_ms,
        datachannel_overhead_ms,
    }
}
