use crate::live_derived::compute_live_endpoint_derived;
use crate::live_schema::{
    LiveArtifactManifestEntryV1, LiveArtifactManifestV1, LiveObservationPairV1,
    ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_A, ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_B,
    ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_A, ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_B,
    ARTIFACT_ROLE_PAIR_REPORT, ARTIFACT_ROLE_PAIR_SUMMARY, CHECKSUM_ALGORITHM,
    LIVE_MANIFEST_SCHEMA_VERSION, LIVE_PAIR_SCHEMA_VERSION,
};
use crate::live_validate::{
    checksum_bytes, count_valid_local_probes, parse_and_validate_live_endpoint,
    verify_live_artifact_manifest, LiveValidationResult, MIN_CLOCK_PROBES, MIN_STATS_SAMPLES,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PairValidationError {
    #[error("endpoint validation failed: {0}")]
    Endpoint(String),
    #[error("duplicate peer role")]
    DuplicateRole,
    #[error("duplicate run id")]
    DuplicateRunId,
    #[error("correlation id mismatch")]
    CorrelationMismatch,
    #[error("software commit mismatch")]
    CommitMismatch,
    #[error("schema version mismatch")]
    SchemaMismatch,
    #[error("observation windows do not overlap")]
    NoOverlap,
    #[error("endpoint not finalized")]
    NotFinalized,
    #[error("insufficient stats samples on {0}")]
    InsufficientStats(String),
    #[error("insufficient valid local clock probes on {0}")]
    InsufficientClockProbes(String),
    #[error("no bidirectional packet progression on {0}")]
    NoPacketProgression(String),
    #[error("checksum mismatch for {0}")]
    ChecksumMismatch(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct PairValidationResult {
    pub valid: bool,
    pub errors: Vec<PairValidationError>,
}

pub struct PairArtifacts {
    pub pair: LiveObservationPairV1,
    pub peer_a: Value,
    pub peer_b: Value,
    pub peer_a_summary: Value,
    pub peer_b_summary: Value,
    pub report_markdown: String,
    pub manifest: LiveArtifactManifestV1,
}

pub fn pair_live_endpoints(
    peer_a_json: &str,
    peer_b_json: &str,
) -> Result<PairArtifacts, PairValidationResult> {
    let mut errors = Vec::new();

    let _peer_a =
        parse_and_validate_live_endpoint(peer_a_json, true).map_err(|r| PairValidationResult {
            valid: false,
            errors: vec![PairValidationError::Endpoint(format!(
                "peer_a: {:?}",
                r.errors
            ))],
        })?;
    let _peer_b =
        parse_and_validate_live_endpoint(peer_b_json, true).map_err(|r| PairValidationResult {
            valid: false,
            errors: vec![PairValidationError::Endpoint(format!(
                "peer_b: {:?}",
                r.errors
            ))],
        })?;

    let a_val: Value = serde_json::from_str(peer_a_json).unwrap();
    let b_val: Value = serde_json::from_str(peer_b_json).unwrap();

    let role_a = a_val.get("peerRole").and_then(|v| v.as_str());
    let role_b = b_val.get("peerRole").and_then(|v| v.as_str());
    if role_a != Some("peer_a") {
        errors.push(PairValidationError::DuplicateRole);
    }
    if role_b != Some("peer_b") {
        errors.push(PairValidationError::DuplicateRole);
    }

    let run_a = a_val.get("runId").and_then(|v| v.as_str());
    let run_b = b_val.get("runId").and_then(|v| v.as_str());
    if run_a.is_some() && run_a == run_b {
        errors.push(PairValidationError::DuplicateRunId);
    }

    let corr_a = a_val.get("sessionCorrelationId").and_then(|v| v.as_str());
    let corr_b = b_val.get("sessionCorrelationId").and_then(|v| v.as_str());
    if corr_a != corr_b {
        errors.push(PairValidationError::CorrelationMismatch);
    }

    let commit_a = a_val.get("softwareCommit").and_then(|v| v.as_str());
    let commit_b = b_val.get("softwareCommit").and_then(|v| v.as_str());
    if commit_a != commit_b {
        errors.push(PairValidationError::CommitMismatch);
    }

    for (_, val) in [("peer_a", &a_val), ("peer_b", &b_val)] {
        if val.get("exportKind").and_then(|v| v.as_str()) != Some("finalized") {
            errors.push(PairValidationError::NotFinalized);
        }
    }

    let overlap = compute_overlap_seconds(&a_val, &b_val);
    if overlap <= 0.0 {
        errors.push(PairValidationError::NoOverlap);
    }

    for (label, val) in [("peer_a", &a_val), ("peer_b", &b_val)] {
        let stats = val
            .get("statsSamples")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        if stats < MIN_STATS_SAMPLES {
            errors.push(PairValidationError::InsufficientStats(label.to_string()));
        }
        let probes = count_valid_local_probes(val);
        if probes < MIN_CLOCK_PROBES {
            errors.push(PairValidationError::InsufficientClockProbes(
                label.to_string(),
            ));
        }
        if !has_packet_progression(val) {
            errors.push(PairValidationError::NoPacketProgression(label.to_string()));
        }
    }

    if !errors.is_empty() {
        return Err(PairValidationResult {
            valid: false,
            errors,
        });
    }

    let corr = corr_a.unwrap().to_string();
    let commit = commit_a.unwrap().to_string();
    let pair_id = format!("pair-{}", &corr[..16]);

    let peer_a_summary = compute_live_endpoint_derived(&a_val);
    let peer_b_summary = compute_live_endpoint_derived(&b_val);

    let pair = LiveObservationPairV1 {
        schema_version: LIVE_PAIR_SCHEMA_VERSION.to_string(),
        pair_id: pair_id.clone(),
        session_correlation_id: corr.clone(),
        software_commit: commit.clone(),
        endpoint_run_ids: vec![
            a_val
                .get("runId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            b_val
                .get("runId")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        ],
        observation_overlap_seconds: overlap,
        per_endpoint_summaries: json!({
            "peerA": peer_a_summary,
            "peerB": peer_b_summary,
        }),
        consistency_checks: json!({
            "sameCorrelationId": true,
            "sameSoftwareCommit": true,
            "overlappingObservation": overlap > 0.0,
            "bothConnected": true,
            "bothRemoteAudio": true,
        }),
        limitations: vec![
            "Exploratory non-authoritative two-peer browser observation.".into(),
            "Not an acoustic mouth-to-ear measurement.".into(),
            "Clock offset is an estimate affected by route asymmetry and scheduling.".into(),
            "No measured one-way network latency.".into(),
            "No transport selection result.".into(),
        ],
        artifact_checksums: None,
    };

    let report = build_pair_report(&pair);
    let manifest = LiveArtifactManifestV1 {
        schema_version: LIVE_MANIFEST_SCHEMA_VERSION.to_string(),
        algorithm: CHECKSUM_ALGORITHM.to_string(),
        pair_id: pair_id.clone(),
        session_correlation_id: corr.clone(),
        software_commit: commit.clone(),
        artifacts: vec![],
    };

    Ok(PairArtifacts {
        pair,
        peer_a: a_val,
        peer_b: b_val,
        peer_a_summary,
        peer_b_summary,
        report_markdown: report,
        manifest,
    })
}

pub fn build_manifest_entries(
    dir: &std::path::Path,
    pair_id: &str,
    corr: &str,
    commit: &str,
) -> LiveArtifactManifestV1 {
    let files = [
        (
            "peer-a.validated.json",
            ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_A,
        ),
        (
            "peer-b.validated.json",
            ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_B,
        ),
        ("peer-a.summary.json", ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_A),
        ("peer-b.summary.json", ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_B),
        ("pair-summary.json", ARTIFACT_ROLE_PAIR_SUMMARY),
        ("report.md", ARTIFACT_ROLE_PAIR_REPORT),
    ];

    let artifacts = files
        .iter()
        .map(|(filename, role)| {
            let bytes = std::fs::read(dir.join(filename)).unwrap_or_default();
            LiveArtifactManifestEntryV1 {
                filename: (*filename).to_string(),
                checksum: checksum_bytes(&bytes),
                artifact_role: (*role).to_string(),
            }
        })
        .collect();

    LiveArtifactManifestV1 {
        schema_version: LIVE_MANIFEST_SCHEMA_VERSION.to_string(),
        algorithm: CHECKSUM_ALGORITHM.to_string(),
        pair_id: pair_id.to_string(),
        session_correlation_id: corr.to_string(),
        software_commit: commit.to_string(),
        artifacts,
    }
}

fn compute_overlap_seconds(a: &Value, b: &Value) -> f64 {
    let (Some(a_start), Some(a_end), Some(b_start), Some(b_end)) = (
        parse_utc(a.get("startedAtUtc")),
        parse_utc(a.get("completedAtUtc")),
        parse_utc(b.get("startedAtUtc")),
        parse_utc(b.get("completedAtUtc")),
    ) else {
        return 0.0;
    };
    let start = a_start.max(b_start);
    let end = a_end.min(b_end);
    (end - start).num_milliseconds().max(0) as f64 / 1000.0
}

fn parse_utc(v: Option<&Value>) -> Option<DateTime<Utc>> {
    v.and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn has_packet_progression(value: &Value) -> bool {
    let samples = value.get("statsSamples").and_then(|v| v.as_array());
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
    let a = first
        .pointer(path)
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let b = last
        .pointer(path)
        .and_then(|v| v.get("value"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    (b - a).max(0.0)
}

fn build_pair_report(pair: &LiveObservationPairV1) -> String {
    format!(
        "Exploratory non-authoritative two-peer browser observation.\n\
Not an acoustic mouth-to-ear measurement.\n\
Not a measured one-way network latency.\n\
Not a transport selection result.\n\n\
## Pair\n\n\
- Pair ID: {}\n\
- Session correlation: {}\n\
- Software commit: {}\n\
- Overlap (s): {:.1}\n\n\
## Limitations\n\n{}\n",
        pair.pair_id,
        pair.session_correlation_id,
        pair.software_commit,
        pair.observation_overlap_seconds,
        pair.limitations
            .iter()
            .map(|l| format!("- {l}"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

pub fn validate_live_directory(dir: &std::path::Path) -> LiveValidationResult {
    verify_live_artifact_manifest(dir)
}
