use crate::live_schema::CLOCK_PROBE_PROTOCOL_VERSION;
use serde_json::Value;
use thiserror::Error;

/// JSON-serialization tolerance for stored vs canonical clock metrics (milliseconds).
pub const CLOCK_METRIC_EPSILON_MS: f64 = 1e-3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CrossDeviceClockMetrics {
    pub rtt_ms: f64,
    pub offset_ms: f64,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ClockMetricError {
    #[error("invalid clock probe sample")]
    InvalidProbe,
    #[error("clock RTT mismatch")]
    RttMismatch,
    #[error("clock offset mismatch")]
    OffsetMismatch,
}

pub fn compute_cross_device_clock_metrics(
    t0: f64,
    t1: f64,
    t2: f64,
    t3: f64,
) -> Option<CrossDeviceClockMetrics> {
    if !t0.is_finite() || !t1.is_finite() || !t2.is_finite() || !t3.is_finite() {
        return None;
    }
    if t3 < t0 || t2 < t1 {
        return None;
    }
    let remote_processing_duration = t2 - t1;
    let local_round_trip_duration = t3 - t0;
    if remote_processing_duration < 0.0 || local_round_trip_duration < 0.0 {
        return None;
    }
    let rtt_ms = local_round_trip_duration - remote_processing_duration;
    if !rtt_ms.is_finite() || rtt_ms < 0.0 {
        return None;
    }
    let offset_ms = (t1 - t0 + (t2 - t3)) / 2.0;
    if !offset_ms.is_finite() {
        return None;
    }
    Some(CrossDeviceClockMetrics { rtt_ms, offset_ms })
}

pub fn stored_clock_metrics_consistent(sample: &Value) -> Result<(), ClockMetricError> {
    let t0 = sample.get("t0").and_then(|v| v.as_f64());
    let t1 = sample.get("t1").and_then(|v| v.as_f64());
    let t2 = sample.get("t2").and_then(|v| v.as_f64());
    let t3 = sample.get("t3").and_then(|v| v.as_f64());
    let (Some(t0), Some(t1), Some(t2), Some(t3)) = (t0, t1, t2, t3) else {
        return Err(ClockMetricError::InvalidProbe);
    };
    let canonical =
        compute_cross_device_clock_metrics(t0, t1, t2, t3).ok_or(ClockMetricError::InvalidProbe)?;
    let stored_rtt = sample
        .get("rttMs")
        .or_else(|| sample.get("rtt_ms"))
        .and_then(|v| v.as_f64())
        .ok_or(ClockMetricError::InvalidProbe)?;
    if !stored_rtt.is_finite() || stored_rtt < 0.0 {
        return Err(ClockMetricError::InvalidProbe);
    }
    if (stored_rtt - canonical.rtt_ms).abs() > CLOCK_METRIC_EPSILON_MS {
        return Err(ClockMetricError::RttMismatch);
    }
    if let Some(stored_offset) = sample
        .get("offsetMs")
        .or_else(|| sample.get("offset_ms"))
        .and_then(|v| v.as_f64())
    {
        if !stored_offset.is_finite() {
            return Err(ClockMetricError::InvalidProbe);
        }
        if (stored_offset - canonical.offset_ms).abs() > CLOCK_METRIC_EPSILON_MS {
            return Err(ClockMetricError::OffsetMismatch);
        }
    }
    Ok(())
}

pub fn probe_requester_role(sample: &Value) -> Option<&str> {
    sample
        .get("requesterRole")
        .or_else(|| sample.get("requester_role"))
        .or_else(|| sample.get("senderRole"))
        .and_then(|v| v.as_str())
}

pub fn probe_responder_role(sample: &Value) -> Option<&str> {
    sample
        .get("responderRole")
        .or_else(|| sample.get("responder_role"))
        .and_then(|v| v.as_str())
}

pub fn opposite_peer_role(role: &str) -> Option<&'static str> {
    match role {
        "peer_a" => Some("peer_b"),
        "peer_b" => Some("peer_a"),
        _ => None,
    }
}

pub fn is_valid_completed_local_probe(sample: &Value, peer_role: &str) -> bool {
    if probe_requester_role(sample) != Some(peer_role) {
        return false;
    }
    if sample.get("timeout").and_then(|v| v.as_bool()) != Some(false) {
        return false;
    }
    if sample.get("duplicate").and_then(|v| v.as_bool()) != Some(false) {
        return false;
    }
    if sample.get("unsolicited").and_then(|v| v.as_bool()) != Some(false) {
        return false;
    }
    if sample.get("invalid").and_then(|v| v.as_bool()) != Some(false) {
        return false;
    }
    let protocol = sample
        .get("protocolVersion")
        .or_else(|| sample.get("protocol_version"))
        .and_then(|v| v.as_i64());
    if protocol != Some(CLOCK_PROBE_PROTOCOL_VERSION) {
        return false;
    }
    let expected_responder = match opposite_peer_role(peer_role) {
        Some(role) => role,
        None => return false,
    };
    if probe_responder_role(sample) != Some(expected_responder) {
        return false;
    }
    let rtt = sample.get("rttMs").or_else(|| sample.get("rtt_ms"));
    if !rtt.map(|v| v.is_number()).unwrap_or(false) {
        return false;
    }
    let t0 = sample.get("t0").and_then(|v| v.as_f64());
    let t1 = sample.get("t1").and_then(|v| v.as_f64());
    let t2 = sample.get("t2").and_then(|v| v.as_f64());
    let t3 = sample.get("t3").and_then(|v| v.as_f64());
    match (t0, t1, t2, t3) {
        (Some(t0), Some(t1), Some(t2), Some(t3)) => {
            compute_cross_device_clock_metrics(t0, t1, t2, t3).is_some()
                && stored_clock_metrics_consistent(sample).is_ok()
        }
        _ => false,
    }
}

pub fn canonical_metrics_from_valid_local_probe(
    sample: &Value,
    peer_role: &str,
) -> Option<CrossDeviceClockMetrics> {
    if !is_valid_completed_local_probe(sample, peer_role) {
        return None;
    }
    let t0 = sample.get("t0").and_then(|v| v.as_f64())?;
    let t1 = sample.get("t1").and_then(|v| v.as_f64())?;
    let t2 = sample.get("t2").and_then(|v| v.as_f64())?;
    let t3 = sample.get("t3").and_then(|v| v.as_f64())?;
    compute_cross_device_clock_metrics(t0, t1, t2, t3)
}
