use crate::live_derived::compute_live_endpoint_derived;
use crate::live_privacy::{contains_live_forbidden_content, scan_live_value_for_forbidden_keys};
use crate::live_schema::{LiveEndpointObservationV1, LIVE_SCHEMA_VERSION};
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

pub const MIN_STATS_SAMPLES: usize = 30;
pub const MIN_CLOCK_PROBES: usize = 10;
pub const MAX_STATS_SAMPLES: usize = 180;
pub const MAX_DURATION_SECONDS: f64 = 180.0;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum LiveValidationError {
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
    #[error("schema version mismatch: expected {LIVE_SCHEMA_VERSION}, got {0}")]
    SchemaVersionMismatch(String),
    #[error("invalid evidence level: {0}")]
    InvalidEvidenceLevel(String),
    #[error("evidence status must be exploratory_non_authoritative")]
    InvalidEvidenceStatus,
    #[error("privacy violation: {0}")]
    PrivacyViolation(String),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("invalid peer role: {0}")]
    InvalidPeerRole(String),
    #[error("invalid session correlation id")]
    InvalidSessionCorrelationId,
    #[error("insufficient stats samples: {0}")]
    InsufficientStatsSamples(usize),
    #[error("insufficient clock probes: {0}")]
    InsufficientClockProbes(usize),
    #[error("observation duration exceeds maximum")]
    ExcessiveDuration,
    #[error("missing limitations")]
    MissingLimitations,
    #[error("connection not connected")]
    NotConnected,
    #[error("remote audio track not reported")]
    NoRemoteAudio,
    #[error("no packet progression")]
    NoPacketProgression,
}

#[derive(Debug, PartialEq, Eq)]
pub struct LiveValidationResult {
    pub valid: bool,
    pub errors: Vec<LiveValidationError>,
}

impl LiveValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
        }
    }

    pub fn err(errors: Vec<LiveValidationError>) -> Self {
        Self {
            valid: false,
            errors,
        }
    }
}

pub fn validate_live_endpoint(json: &str, require_finalized: bool) -> LiveValidationResult {
    let mut errors = Vec::new();

    if let Some(violation) = contains_live_forbidden_content(json) {
        errors.push(LiveValidationError::PrivacyViolation(violation));
    }

    let value: Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => {
            return LiveValidationResult::err(vec![LiveValidationError::InvalidJson(e.to_string())])
        }
    };

    if let Some(violation) = scan_live_value_for_forbidden_keys(&value, "") {
        errors.push(LiveValidationError::PrivacyViolation(violation));
    }

    let schema = value
        .get("schemaVersion")
        .or_else(|| value.get("schema_version"))
        .and_then(|v| v.as_str());
    match schema {
        Some(v) if v == LIVE_SCHEMA_VERSION => {}
        Some(v) => errors.push(LiveValidationError::SchemaVersionMismatch(v.to_string())),
        None => errors.push(LiveValidationError::MissingField("schemaVersion".into())),
    }

    let level = value
        .get("evidenceLevel")
        .or_else(|| value.get("evidence_level"))
        .and_then(|v| v.as_str());
    match level {
        Some("browser_network_observation") => {}
        Some(v) => errors.push(LiveValidationError::InvalidEvidenceLevel(v.to_string())),
        None => errors.push(LiveValidationError::MissingField("evidenceLevel".into())),
    }

    let status = value
        .get("evidenceStatus")
        .or_else(|| value.get("evidence_status"))
        .and_then(|v| v.as_str());
    match status {
        Some("exploratory_non_authoritative") => {}
        _ => errors.push(LiveValidationError::InvalidEvidenceStatus),
    }

    let role = value
        .get("peerRole")
        .or_else(|| value.get("peer_role"))
        .and_then(|v| v.as_str());
    match role {
        Some("peer_a") | Some("peer_b") => {}
        Some(v) => errors.push(LiveValidationError::InvalidPeerRole(v.to_string())),
        None => errors.push(LiveValidationError::MissingField("peerRole".into())),
    }

    let corr = value
        .get("sessionCorrelationId")
        .or_else(|| value.get("session_correlation_id"))
        .and_then(|v| v.as_str());
    if !is_valid_correlation_id(corr) {
        errors.push(LiveValidationError::InvalidSessionCorrelationId);
    }

    let limitations = value.get("limitations").and_then(|v| v.as_array());
    if limitations.map(|a| a.is_empty()).unwrap_or(true) {
        errors.push(LiveValidationError::MissingLimitations);
    }

    if require_finalized {
        validate_finalized_requirements(&value, &mut errors);
    }

    if errors.is_empty() {
        LiveValidationResult::ok()
    } else {
        LiveValidationResult::err(errors)
    }
}

fn validate_software_commit(value: &Value, errors: &mut Vec<LiveValidationError>) {
    let commit = value.get("softwareCommit").and_then(|v| v.as_str());
    match commit {
        Some(c) if c.len() >= 7 && c != "unknown" && !c.contains("placeholder") => {}
        _ => errors.push(LiveValidationError::MissingField("softwareCommit".into())),
    }
}

fn is_valid_correlation_id(corr: Option<&str>) -> bool {
    match corr {
        Some(id) if id.len() == 32 => id.chars().all(|c| c.is_ascii_hexdigit()),
        _ => false,
    }
}

fn validate_finalized_requirements(value: &Value, errors: &mut Vec<LiveValidationError>) {
    validate_software_commit(value, errors);
    let stats_len = value
        .get("statsSamples")
        .or_else(|| value.get("stats_samples"))
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if stats_len < MIN_STATS_SAMPLES {
        errors.push(LiveValidationError::InsufficientStatsSamples(stats_len));
    }
    if stats_len > MAX_STATS_SAMPLES {
        errors.push(LiveValidationError::ExcessiveDuration);
    }

    let probe_len = value
        .get("clockProbes")
        .or_else(|| value.get("clock_probes"))
        .and_then(|v| v.get("samples"))
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    if probe_len < MIN_CLOCK_PROBES {
        errors.push(LiveValidationError::InsufficientClockProbes(probe_len));
    }

    if let (Some(start), Some(end)) = (
        parse_utc(value.get("startedAtUtc").or(value.get("started_at_utc"))),
        parse_utc(
            value
                .get("completedAtUtc")
                .or(value.get("completed_at_utc")),
        ),
    ) {
        let dur = (end - start).num_milliseconds() as f64 / 1000.0;
        if dur > MAX_DURATION_SECONDS {
            errors.push(LiveValidationError::ExcessiveDuration);
        }
    }

    let conn_state = value
        .pointer("/connectionLifecycle/peerConnectionState")
        .or(value.pointer("/connection_lifecycle/peer_connection_state"))
        .and_then(|v| v.as_str());
    if conn_state != Some("connected") {
        errors.push(LiveValidationError::NotConnected);
    }

    let remote_track = value
        .pointer("/playout/remoteAudioTrackReceived")
        .or(value.pointer("/playout/remote_audio_track_received"))
        .and_then(|v| v.as_bool());
    if remote_track != Some(true) {
        errors.push(LiveValidationError::NoRemoteAudio);
    }

    if !has_packet_progression(value) {
        errors.push(LiveValidationError::NoPacketProgression);
    }
}

fn parse_utc(v: Option<&Value>) -> Option<DateTime<Utc>> {
    v.and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn has_packet_progression(value: &Value) -> bool {
    let samples = value
        .get("statsSamples")
        .or_else(|| value.get("stats_samples"))
        .and_then(|v| v.as_array());
    let Some(samples) = samples else {
        return false;
    };
    if samples.len() < 2 {
        return false;
    }
    let first = &samples[0];
    let last = &samples[samples.len() - 1];
    let inbound_delta = counter_delta(first, last, "/inboundAudio/packetsReceived");
    let outbound_delta = counter_delta(first, last, "/outboundAudio/packetsSent");
    inbound_delta > 0.0 && outbound_delta > 0.0
}

fn counter_delta(first: &Value, last: &Value, path: &str) -> f64 {
    let a = first.pointer(path).and_then(metric_num).unwrap_or(0.0);
    let b = last.pointer(path).and_then(metric_num).unwrap_or(0.0);
    (b - a).max(0.0)
}

fn metric_num(v: &Value) -> Option<f64> {
    v.get("value")
        .and_then(|v| v.as_f64())
        .or_else(|| v.as_f64())
}

pub fn parse_and_validate_live_endpoint(
    json: &str,
    require_finalized: bool,
) -> Result<LiveEndpointObservationV1, LiveValidationResult> {
    let result = validate_live_endpoint(json, require_finalized);
    if !result.valid {
        return Err(result);
    }
    let mut endpoint: LiveEndpointObservationV1 = serde_json::from_str(json).map_err(|e| {
        LiveValidationResult::err(vec![LiveValidationError::InvalidJson(e.to_string())])
    })?;
    let value: Value = serde_json::from_str(json).unwrap();
    endpoint.derived = Some(compute_live_endpoint_derived(&value));
    Ok(endpoint)
}

pub fn checksum_json(json: &str) -> String {
    blake3::hash(json.as_bytes()).to_hex().to_string()
}
