use crate::live_clock::{
    compute_cross_device_clock_metrics, is_valid_completed_local_probe, opposite_peer_role,
    probe_requester_role, probe_responder_role, stored_clock_metrics_consistent, ClockMetricError,
};
use crate::live_derived::compute_live_endpoint_derived;
use crate::live_privacy::{contains_live_forbidden_content, scan_live_value_for_forbidden_keys};
use crate::live_schema::{
    required_live_artifact_spec, LiveEndpointObservationV1, LiveMetricValue,
    ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_A, ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_B,
    ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_A, ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_B,
    ARTIFACT_ROLE_PAIR_REPORT, ARTIFACT_ROLE_PAIR_SUMMARY, CHECKSUM_ALGORITHM,
    LIVE_MANIFEST_SCHEMA_VERSION, LIVE_PAIR_REPORT_DISCLAIMER_PREFIX, LIVE_PAIR_SCHEMA_VERSION,
    LIVE_SCHEMA_VERSION,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use thiserror::Error;

pub const MIN_STATS_SAMPLES: usize = 30;
pub const MIN_CLOCK_PROBES: usize = 10;
pub const MAX_STATS_SAMPLES: usize = 180;
pub const MAX_DURATION_SECONDS: f64 = 180.0;
pub use crate::live_clock::CrossDeviceClockMetrics;
pub use crate::live_clock::CLOCK_METRIC_EPSILON_MS;

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
    #[error("clock RTT mismatch")]
    ClockRttMismatch,
    #[error("clock offset mismatch")]
    ClockOffsetMismatch,
    #[error("invalid responder role: {0}")]
    InvalidResponderRole(String),
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
    #[error("no inbound RTP audio progression")]
    NoInboundRtpAudioProgression,
    #[error("no outbound RTP audio progression")]
    NoOutboundRtpAudioProgression,
    #[error("invalid audio counter provenance")]
    InvalidAudioCounterProvenance,
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

pub fn validate_cross_device_clock_timestamps(t0: f64, t1: f64, t2: f64, t3: f64) -> Option<f64> {
    compute_cross_device_clock_metrics(t0, t1, t2, t3).map(|metrics| metrics.rtt_ms)
}

fn map_clock_metric_error(err: ClockMetricError) -> LiveValidationError {
    match err {
        ClockMetricError::InvalidProbe => LiveValidationError::InvalidClockProbe,
        ClockMetricError::RttMismatch => LiveValidationError::ClockRttMismatch,
        ClockMetricError::OffsetMismatch => LiveValidationError::ClockOffsetMismatch,
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

    validate_audio_counter_provenance(value, errors);
    if !has_inbound_rtp_audio_progression(value) {
        errors.push(LiveValidationError::NoInboundRtpAudioProgression);
    }
    if !has_outbound_rtp_audio_progression(value) {
        errors.push(LiveValidationError::NoOutboundRtpAudioProgression);
    }
}

fn validate_clock_probe_samples(value: &Value, errors: &mut Vec<LiveValidationError>) {
    let peer_role = value
        .get("peerRole")
        .or_else(|| value.get("peer_role"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let expected_responder = opposite_peer_role(peer_role);
    let samples = value
        .pointer("/clockProbes/samples")
        .or_else(|| value.pointer("/clock_probes/samples"))
        .and_then(|v| v.as_array());
    let Some(samples) = samples else {
        errors.push(LiveValidationError::InvalidClockProbe);
        return;
    };
    for sample in samples {
        let requester = probe_requester_role(sample);
        let is_completed = sample.get("timeout").and_then(|v| v.as_bool()) == Some(false)
            && sample.get("duplicate").and_then(|v| v.as_bool()) == Some(false)
            && sample.get("unsolicited").and_then(|v| v.as_bool()) == Some(false)
            && sample.get("invalid").and_then(|v| v.as_bool()) == Some(false)
            && sample
                .get("rttMs")
                .or_else(|| sample.get("rtt_ms"))
                .is_some();
        if !is_completed {
            continue;
        }
        if requester == Some(peer_role) {
            let responder = probe_responder_role(sample);
            if responder.is_none() {
                errors.push(LiveValidationError::InvalidResponderRole(
                    "missing responderRole".into(),
                ));
                return;
            }
            if responder == requester {
                errors.push(LiveValidationError::InvalidResponderRole(
                    "responder equals requester".into(),
                ));
                return;
            }
            if expected_responder.is_some() && responder != expected_responder {
                errors.push(LiveValidationError::InvalidResponderRole(
                    responder.unwrap_or("unknown").into(),
                ));
                return;
            }
        }
        if requester == Some(peer_role)
            && sample.get("invalid").and_then(|v| v.as_bool()) == Some(true)
        {
            errors.push(LiveValidationError::InvalidClockProbe);
            return;
        }
        if requester == Some(peer_role) && is_completed {
            if let Err(err) = stored_clock_metrics_consistent(sample) {
                errors.push(map_clock_metric_error(err));
                return;
            }
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

pub const RTP_AUDIO_COUNTER_SOURCE: &str = "rtp_audio";

fn validate_audio_counter_provenance(value: &Value, errors: &mut Vec<LiveValidationError>) {
    let samples = value
        .get("statsSamples")
        .or_else(|| value.get("stats_samples"))
        .and_then(|v| v.as_array());
    let Some(samples) = samples else {
        return;
    };
    for sample in samples {
        for key in ["inboundAudio", "outboundAudio"] {
            let Some(audio) = sample.get(key) else {
                continue;
            };
            if audio.as_object().is_none_or(|o| o.is_empty()) {
                continue;
            }
            let source = audio
                .get("counterSource")
                .and_then(|v| v.get("value"))
                .and_then(|v| v.as_str());
            if source != Some(RTP_AUDIO_COUNTER_SOURCE) {
                errors.push(LiveValidationError::InvalidAudioCounterProvenance);
                return;
            }
        }
    }
}

pub fn has_inbound_rtp_audio_progression(value: &Value) -> bool {
    has_direction_rtp_audio_progression(value, "inboundAudio", "/inboundAudio/packetsReceived")
}

pub fn has_outbound_rtp_audio_progression(value: &Value) -> bool {
    has_direction_rtp_audio_progression(value, "outboundAudio", "/outboundAudio/packetsSent")
}

pub fn has_rtp_audio_packet_progression(value: &Value) -> bool {
    has_inbound_rtp_audio_progression(value) && has_outbound_rtp_audio_progression(value)
}

fn has_direction_rtp_audio_progression(value: &Value, audio_key: &str, packet_path: &str) -> bool {
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
    let valid: Vec<&Value> = samples
        .iter()
        .filter(|sample| has_rtp_audio_provenance(sample, audio_key))
        .filter(|sample| sample.pointer(packet_path).and_then(metric_num).is_some())
        .collect();
    if valid.len() < 2 {
        return false;
    }
    counter_delta(valid[0], valid[valid.len() - 1], packet_path) > 0.0
}

fn has_rtp_audio_provenance(sample: &Value, audio_key: &str) -> bool {
    sample
        .get(audio_key)
        .and_then(|audio| audio.get("counterSource"))
        .and_then(|source| source.get("value"))
        .and_then(|value| value.as_str())
        == Some(RTP_AUDIO_COUNTER_SOURCE)
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

    if artifacts.is_empty() {
        return LiveValidationResult::err(vec![LiveValidationError::ManifestVerification(
            "empty artifacts array".into(),
        )]);
    }

    let mut errors = Vec::new();
    let mut seen_filenames = std::collections::HashSet::new();
    let mut seen_roles = std::collections::HashSet::new();
    let mut manifest_entries = std::collections::HashMap::new();

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

        if let Some(reason) = validate_manifest_filename(filename) {
            errors.push(LiveValidationError::ManifestVerification(format!(
                "{filename}: {reason}"
            )));
            continue;
        }

        if !seen_filenames.insert(filename.to_string()) {
            errors.push(LiveValidationError::ManifestVerification(format!(
                "duplicate filename: {filename}"
            )));
        }
        if !seen_roles.insert(role.to_string()) {
            errors.push(LiveValidationError::ManifestVerification(format!(
                "duplicate artifactRole: {role}"
            )));
        }

        manifest_entries.insert(filename.to_string(), role.to_string());

        let path = dir.join(filename);
        if !path.starts_with(dir) {
            errors.push(LiveValidationError::ManifestVerification(format!(
                "manifest entry escapes evidence directory: {filename}"
            )));
            continue;
        }

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

        match role {
            ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_A | ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_B => {
                let json = String::from_utf8_lossy(&bytes);
                let result = validate_live_endpoint(&json, true);
                if !result.valid {
                    errors.extend(result.errors.into_iter().map(|e| {
                        LiveValidationError::ManifestVerification(format!("{filename}: {e}"))
                    }));
                }
            }
            ARTIFACT_ROLE_PAIR_SUMMARY => {
                let parsed: Result<Value, _> = serde_json::from_slice(&bytes);
                match parsed {
                    Ok(value) => {
                        let pair_schema = value.get("schemaVersion").and_then(|v| v.as_str());
                        if pair_schema != Some(LIVE_PAIR_SCHEMA_VERSION) {
                            errors.push(LiveValidationError::ManifestVerification(format!(
                                "{filename}: pair schema mismatch"
                            )));
                        }
                    }
                    Err(e) => errors.push(LiveValidationError::ManifestVerification(format!(
                        "{filename}: invalid JSON: {e}"
                    ))),
                }
            }
            ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_A | ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_B => {
                if serde_json::from_slice::<Value>(&bytes).is_err() {
                    errors.push(LiveValidationError::ManifestVerification(format!(
                        "{filename}: invalid JSON"
                    )));
                }
            }
            ARTIFACT_ROLE_PAIR_REPORT => {
                let text = String::from_utf8_lossy(&bytes);
                if !text.starts_with(LIVE_PAIR_REPORT_DISCLAIMER_PREFIX) {
                    errors.push(LiveValidationError::ManifestVerification(format!(
                        "{filename}: missing required disclaimer prefix"
                    )));
                }
            }
            other => errors.push(LiveValidationError::ManifestVerification(format!(
                "unexpected artifactRole: {other}"
            ))),
        }
    }

    for (required_filename, required_role) in required_live_artifact_spec() {
        match manifest_entries.get(*required_filename) {
            Some(role) if role == required_role => {}
            Some(role) => errors.push(LiveValidationError::ManifestVerification(format!(
                "unexpected artifactRole for {required_filename}: {role}"
            ))),
            None => errors.push(LiveValidationError::ManifestVerification(format!(
                "missing required manifest entry: {required_filename}"
            ))),
        }
    }

    for filename in manifest_entries.keys() {
        if !required_live_artifact_spec()
            .iter()
            .any(|(required, _)| required == filename)
        {
            errors.push(LiveValidationError::ManifestVerification(format!(
                "unexpected manifest filename: {filename}"
            )));
        }
    }

    if errors.is_empty() {
        LiveValidationResult::ok()
    } else {
        LiveValidationResult::err(errors)
    }
}

pub fn validate_manifest_filename(filename: &str) -> Option<&'static str> {
    if filename.is_empty() {
        return Some("empty filename");
    }
    if filename.starts_with('/') || filename.starts_with('\\') {
        return Some("absolute path not allowed");
    }
    if filename.contains("..") {
        return Some("path traversal not allowed");
    }
    if filename.contains('/') || filename.contains('\\') {
        return Some("path separators not allowed");
    }
    None
}
