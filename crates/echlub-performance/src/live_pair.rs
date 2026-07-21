use crate::live_derived::compute_live_endpoint_derived;
use crate::live_schema::{LiveObservationPairV1, LIVE_PAIR_SCHEMA_VERSION};
use crate::live_validate::{
    checksum_json, parse_and_validate_live_endpoint, LiveValidationError, LiveValidationResult,
    MIN_CLOCK_PROBES, MIN_STATS_SAMPLES,
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
    #[error("correlation id mismatch")]
    CorrelationMismatch,
    #[error("software commit mismatch")]
    CommitMismatch,
    #[error("schema version mismatch")]
    SchemaMismatch,
    #[error("observation windows do not overlap")]
    NoOverlap,
    #[error("insufficient stats samples on {0}")]
    InsufficientStats(String),
    #[error("insufficient clock probes on {0}")]
    InsufficientClockProbes(String),
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
    pub manifest: Value,
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
        let probes = val
            .pointer("/clockProbes/samples")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        if probes < MIN_CLOCK_PROBES {
            errors.push(PairValidationError::InsufficientClockProbes(
                label.to_string(),
            ));
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

    let checksum_a = checksum_json(peer_a_json);
    let checksum_b = checksum_json(peer_b_json);

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
        artifact_checksums: json!({
            "peerA": checksum_a,
            "peerB": checksum_b,
        }),
    };

    let report = build_pair_report(&pair);
    let manifest = json!({
        "pairId": pair_id,
        "sessionCorrelationId": corr,
        "softwareCommit": commit,
        "files": [
            "peer-a.validated.json",
            "peer-b.validated.json",
            "peer-a.summary.json",
            "peer-b.summary.json",
            "pair-summary.json",
            "report.md",
            "artifact-manifest.json",
        ],
        "checksums": {
            "peerA": checksum_a,
            "peerB": checksum_b,
        },
    });

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
    let mut errors = Vec::new();
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                count += 1;
                let json = std::fs::read_to_string(&path).unwrap_or_default();
                let result = validate_live_endpoint_file(&json, false);
                if !result.valid {
                    errors.extend(result.errors.into_iter().map(|e| {
                        LiveValidationError::InvalidJson(format!("{}: {e}", path.display()))
                    }));
                }
            }
        }
    }
    if count == 0 {
        errors.push(LiveValidationError::MissingField(
            "no json artifacts".into(),
        ));
    }
    if errors.is_empty() {
        LiveValidationResult::ok()
    } else {
        LiveValidationResult::err(errors)
    }
}

fn validate_live_endpoint_file(json: &str, require_finalized: bool) -> LiveValidationResult {
    crate::live_validate::validate_live_endpoint(json, require_finalized)
}
