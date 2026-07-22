use crate::live_derived::compute_live_endpoint_derived;
use crate::live_privacy::{contains_live_forbidden_content, scan_live_value_for_forbidden_keys};
use crate::live_schema::{
    LiveEndpointObservationV1, LiveMetricValue, CHECKSUM_ALGORITHM, CLOCK_PROBE_PROTOCOL_VERSION,
    LIVE_MANIFEST_SCHEMA_VERSION, LIVE_SCHEMA_VERSION,
};
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
    #[error("invalid export kind: {0}")]
    InvalidExportKind(String),
    #[error("invalid software commit")]
    InvalidSoftwareCommit,
    #[error("invalid timestamps")]
    InvalidTimestamps,
    #[error("insufficient stats samples: {0}")]
    InsufficientStatsSamples(usize),
    #[error("insufficient valid local clock probes: {0}")]
    InsufficientClockProbes(usize),
    #[error("observation duration exceeds maximum")]
    ExcessiveDuration,
    #[error("stats offsets not strictly monotonic")]
    StatsOffsetsNotMonotonic,
    #[error("stats offsets outside observation duration")]
    StatsOffsetsOutOfBounds,
    #[error("invalid clock probe sample")]
    InvalidClockProbe,
    #[error("missing limitations")]
    MissingLimitations,
    #[error("connection not connected")]
    NotConnected,
    #[error("ICE not connected")]
    IceNotConnected,
    #[error("signaling not stable")]
    SignalingNotStable,
    #[error("data channel not open")]
    DataChannelNotOpen,
    #[error("remote audio track not reported")]
    NoRemoteAudio,
    #[error("remote audio track not live")]
    RemoteAudioNotLive,
    #[error("no packet progression")]
    NoPacketProgression,
    #[error("manifest schema mismatch: {0}")]
    ManifestSchemaMismatch(String),
    #[error("checksum mismatch for {0}")]
    ChecksumMismatch(String),
    #[error("manifest verification failed: {0}")]
    ManifestVerification(String),
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

pub fn count_valid_local_probes(value: &Value) -> usize {
    let peer_role = value
        .get("peerRole")
        .or_else(|| value.get("peer_role"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let samples = value
        .pointer("/clockProbes/samples")
        .or_else(|| value.pointer("/clock_probes/samples"))
        .and_then(|v| v.as_array());
    let Some(samples) = samples else {
        return 0;
    };
    samples
        .iter()
        .filter(|sample| is_valid_completed_local_probe(sample, peer_role))
        .count()
}

fn is_valid_completed_local_probe(sample: &Value, peer_role: &str) -> bool {
    let requester = sample
        .get("senderRole")
        .or_else(|| sample.get("requesterRole"))
        .or_else(|| sample.get("requester_role"))
        .and_then(|v| v.as_str());
    if requester != Some(peer_role) {
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
    let rtt = sample.get("rttMs").or_else(|| sample.get("rtt_ms"));
    if !rtt.map(|v| v.is_number()).unwrap_or(false) {
        return false;
    }
    let t0 = sample.get("t0").and_then(|v| v.as_f64());
    let t1 = sample.get("t1").and_then(|v| v.as_f64());
    let t2 = sample.get("t2").and_then(|v| v.as_f64());
    let t3 = sample.get("t3").and_then(|v| v.as_f64());
    match (t0, t1, t2, t3) {
        (Some(t0), Some(t1), Some(t2), Some(t3))
            if t0.is_finite() && t1.is_finite() && t2.is_finite() && t3.is_finite() =>
        {
            if !(t0 <= t1 && t1 <= t2 && t2 <= t3) {
                return false;
            }
            let rtt_val = rtt.and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
            if !rtt_val.is_finite() || rtt_val < 0.0 {
                return false;
            }
            true
        }
        _ => false,
    }
}

fn validate_finalized_requirements(value: &Value, errors: &mut Vec<LiveValidationError>) {
    let export_kind = value
        .get("exportKind")
        .or_else(|| value.get("export_kind"))
        .and_then(|v| v.as_str());
    if export_kind != Some("finalized") {
        errors.push(LiveValidationError::InvalidExportKind(
            export_kind.unwrap_or("missing").into(),
        ));
    }

    let commit = value.get("softwareCommit").and_then(|v| v.as_str());
    if !commit
        .map(|c| c.len() == 40 && c.chars().all(|ch| ch.is_ascii_hexdigit()))
        .unwrap_or(false)
    {
        errors.push(LiveValidationError::InvalidSoftwareCommit);
    }

    let start = parse_utc(value.get("startedAtUtc").or(value.get("started_at_utc")));
    let end = parse_utc(
        value
            .get("completedAtUtc")
            .or(value.get("completed_at_utc")),
    );
    if let (Some(start), Some(end)) = (start, end) {
        if end <= start {
            errors.push(LiveValidationError::InvalidTimestamps);
        } else {
            let dur = (end - start).num_milliseconds() as f64 / 1000.0;
            if dur <= 0.0 || dur > MAX_DURATION_SECONDS {
                errors.push(LiveValidationError::ExcessiveDuration);
            }
            validate_stats_samples(value, dur, errors);
        }
    } else {
        errors.push(LiveValidationError::InvalidTimestamps);
    }

    let valid_probes = count_valid_local_probes(value);
    if valid_probes < MIN_CLOCK_PROBES {
        errors.push(LiveValidationError::InsufficientClockProbes(valid_probes));
    }

    validate_clock_probe_samples(value, errors);

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

    let conn_state = value
        .pointer("/connectionLifecycle/peerConnectionState")
        .or(value.pointer("/connection_lifecycle/peer_connection_state"))
        .and_then(|v| v.as_str());
    if conn_state != Some("connected") {
        errors.push(LiveValidationError::NotConnected);
    }

    let ice_state = value
        .pointer("/connectionLifecycle/iceConnectionState")
        .or(value.pointer("/connection_lifecycle/ice_connection_state"))
        .and_then(|v| v.as_str());
    if ice_state != Some("connected") && ice_state != Some("completed") {
        errors.push(LiveValidationError::IceNotConnected);
    }

    let signaling_state = value
        .pointer("/connectionLifecycle/signalingState")
        .or(value.pointer("/connection_lifecycle/signaling_state"))
        .and_then(|v| v.as_str());
    if signaling_state != Some("stable") {
        errors.push(LiveValidationError::SignalingNotStable);
    }

    let dc_state = value
        .pointer("/dataChannel/readyState")
        .or(value.pointer("/data_channel/ready_state"))
        .and_then(|v| v.as_str());
    if dc_state != Some("open") {
        errors.push(LiveValidationError::DataChannelNotOpen);
    }

    let remote_track = value
        .pointer("/playout/remoteAudioTrackReceived")
        .or(value.pointer("/playout/remote_audio_track_received"))
        .and_then(|v| v.as_bool());
    if remote_track != Some(true) {
        errors.push(LiveValidationError::NoRemoteAudio);
    }

    let remote_ready = value
        .pointer("/playout/remoteAudioTrackReadyState")
        .or(value.pointer("/playout/remote_audio_track_ready_state"))
        .and_then(|v| v.as_str());
    if remote_ready != Some("live") {
        errors.push(LiveValidationError::RemoteAudioNotLive);
    }

    if !has_packet_progression(value) {
        errors.push(LiveValidationError::NoPacketProgression);
    }
}

fn validate_clock_probe_samples(value: &Value, errors: &mut Vec<LiveValidationError>) {
    let peer_role = value
        .get("peerRole")
        .or_else(|| value.get("peer_role"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let samples = value
        .pointer("/clockProbes/samples")
        .or_else(|| value.pointer("/clock_probes/samples"))
        .and_then(|v| v.as_array());
    let Some(samples) = samples else {
        errors.push(LiveValidationError::InvalidClockProbe);
        return;
    };
    for sample in samples {
        let requester = sample
            .get("senderRole")
            .or_else(|| sample.get("requesterRole"))
            .and_then(|v| v.as_str());
        if requester == Some(peer_role)
            && sample.get("invalid").and_then(|v| v.as_bool()) == Some(true)
        {
            errors.push(LiveValidationError::InvalidClockProbe);
            break;
        }
    }
}

fn validate_stats_samples(value: &Value, duration_ms: f64, errors: &mut Vec<LiveValidationError>) {
    let duration_ms = duration_ms * 1000.0;
    let samples = value
        .get("statsSamples")
        .or_else(|| value.get("stats_samples"))
        .and_then(|v| v.as_array());
    let Some(samples) = samples else {
        return;
    };
    let mut prev: Option<f64> = None;
    for sample in samples {
        let offset = sample
            .get("offsetMs")
            .or_else(|| sample.get("offset_ms"))
            .and_then(|v| v.as_f64());
        let Some(offset) = offset else {
            errors.push(LiveValidationError::StatsOffsetsNotMonotonic);
            return;
        };
        if !offset.is_finite() {
            errors.push(LiveValidationError::StatsOffsetsNotMonotonic);
            return;
        }
        if let Some(prev_offset) = prev {
            if offset <= prev_offset {
                errors.push(LiveValidationError::StatsOffsetsNotMonotonic);
                return;
            }
        }
        if offset < 0.0 || offset > duration_ms {
            errors.push(LiveValidationError::StatsOffsetsOutOfBounds);
            return;
        }
        prev = Some(offset);
    }
}

fn is_valid_correlation_id(corr: Option<&str>) -> bool {
    match corr {
        Some(id) if id.len() == 32 => id.chars().all(|c| c.is_ascii_hexdigit()),
        _ => false,
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
    if let Ok(parsed) = serde_json::from_value::<LiveMetricValue>(v.clone()) {
        return parsed.as_f64();
    }
    v.get("value")
        .and_then(|v| v.as_f64())
        .or_else(|| v.as_f64())
}

impl LiveMetricValue {
    fn as_f64(&self) -> Option<f64> {
        match self {
            LiveMetricValue::ObservedNumber { value } => Some(*value),
            _ => None,
        }
    }
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
    checksum_bytes(json.as_bytes())
}

pub fn checksum_bytes(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

pub fn verify_live_artifact_manifest(dir: &std::path::Path) -> LiveValidationResult {
    let manifest_path = dir.join("artifact-manifest.json");
    let manifest_json = match std::fs::read_to_string(&manifest_path) {
        Ok(json) => json,
        Err(e) => {
            return LiveValidationResult::err(vec![LiveValidationError::ManifestVerification(
                format!("missing artifact-manifest.json: {e}"),
            )])
        }
    };

    let manifest: Value = match serde_json::from_str(&manifest_json) {
        Ok(v) => v,
        Err(e) => {
            return LiveValidationResult::err(vec![LiveValidationError::ManifestVerification(
                e.to_string(),
            )])
        }
    };

    let schema = manifest.get("schemaVersion").and_then(|v| v.as_str());
    if schema != Some(LIVE_MANIFEST_SCHEMA_VERSION) {
        return LiveValidationResult::err(vec![LiveValidationError::ManifestSchemaMismatch(
            schema.unwrap_or("missing").into(),
        )]);
    }

    let algorithm = manifest.get("algorithm").and_then(|v| v.as_str());
    if algorithm != Some(CHECKSUM_ALGORITHM) {
        return LiveValidationResult::err(vec![LiveValidationError::ManifestVerification(
            format!(
                "unexpected checksum algorithm: {}",
                algorithm.unwrap_or("missing")
            ),
        )]);
    }

    let artifacts = manifest.get("artifacts").and_then(|v| v.as_array());
    let Some(artifacts) = artifacts else {
        return LiveValidationResult::err(vec![LiveValidationError::ManifestVerification(
            "missing artifacts array".into(),
        )]);
    };

    let mut errors = Vec::new();
    for entry in artifacts {
        let filename = entry.get("filename").and_then(|v| v.as_str());
        let expected = entry.get("checksum").and_then(|v| v.as_str());
        let role = entry.get("artifactRole").and_then(|v| v.as_str());
        let (Some(filename), Some(expected), Some(role)) = (filename, expected, role) else {
            errors.push(LiveValidationError::ManifestVerification(
                "manifest entry missing filename/checksum/artifactRole".into(),
            ));
            continue;
        };

        let path = dir.join(filename);
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                errors.push(LiveValidationError::ManifestVerification(format!(
                    "{filename}: {e}"
                )));
                continue;
            }
        };
        let actual = checksum_bytes(&bytes);
        if actual != expected {
            errors.push(LiveValidationError::ChecksumMismatch(filename.to_string()));
        }

        if role.ends_with("validated-peer-a") || role.ends_with("validated-peer-b") {
            let json = String::from_utf8_lossy(&bytes);
            let result = validate_live_endpoint(&json, true);
            if !result.valid {
                errors.extend(result.errors.into_iter().map(|e| {
                    LiveValidationError::ManifestVerification(format!("{filename}: {e}"))
                }));
            }
        }
    }

    if errors.is_empty() {
        LiveValidationResult::ok()
    } else {
        LiveValidationResult::err(errors)
    }
}
