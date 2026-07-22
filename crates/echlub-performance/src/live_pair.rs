use crate::live_derived::compute_live_endpoint_derived;
use crate::live_schema::{
    required_live_artifact_spec, LiveArtifactManifestEntryV1, LiveArtifactManifestV1,
    LiveObservationPairV1, CHECKSUM_ALGORITHM, LIVE_MANIFEST_SCHEMA_VERSION,
    LIVE_PAIR_SCHEMA_VERSION,
};
use crate::live_validate::{
    checksum_bytes, count_valid_local_probes, has_rtp_audio_packet_progression,
    parse_and_validate_live_endpoint, verify_live_artifact_manifest, LiveValidationError,
    LiveValidationResult, MIN_CLOCK_PROBES, MIN_STATS_SAMPLES,
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
        if !has_rtp_audio_packet_progression(val) {
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ManifestBuildError {
    #[error("missing required artifact file: {0}")]
    MissingFile(String),
    #[error("unsafe manifest filename: {0}")]
    UnsafeFilename(String),
}

pub fn build_manifest_entries(
    dir: &std::path::Path,
    pair_id: &str,
    corr: &str,
    commit: &str,
) -> Result<LiveArtifactManifestV1, ManifestBuildError> {
    let mut artifacts = Vec::with_capacity(required_live_artifact_spec().len());

    for (filename, role) in required_live_artifact_spec() {
        if let Some(reason) = crate::live_validate::validate_manifest_filename(filename) {
            return Err(ManifestBuildError::UnsafeFilename(format!(
                "{filename}: {reason}"
            )));
        }
        let path = dir.join(filename);
        let bytes = std::fs::read(&path)
            .map_err(|_| ManifestBuildError::MissingFile(filename.to_string()))?;
        artifacts.push(LiveArtifactManifestEntryV1 {
            filename: (*filename).to_string(),
            checksum: checksum_bytes(&bytes),
            artifact_role: (*role).to_string(),
        });
    }

    Ok(LiveArtifactManifestV1 {
        schema_version: LIVE_MANIFEST_SCHEMA_VERSION.to_string(),
        algorithm: CHECKSUM_ALGORITHM.to_string(),
        pair_id: pair_id.to_string(),
        session_correlation_id: corr.to_string(),
        software_commit: commit.to_string(),
        artifacts,
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

fn json_semantic_equal(actual: &Value, expected: &Value) -> bool {
    actual == expected
}

fn verify_live_directory_semantics(dir: &std::path::Path) -> LiveValidationResult {
    let mut errors = Vec::new();

    let read_required = |filename: &str| -> Result<String, LiveValidationError> {
        std::fs::read_to_string(dir.join(filename))
            .map_err(|e| LiveValidationError::ManifestVerification(format!("{filename}: {e}")))
    };

    let peer_a_json = match read_required("peer-a.validated.json") {
        Ok(json) => json,
        Err(err) => return LiveValidationResult::err(vec![err]),
    };
    let peer_b_json = match read_required("peer-b.validated.json") {
        Ok(json) => json,
        Err(err) => return LiveValidationResult::err(vec![err]),
    };

    let expected = match pair_live_endpoints(&peer_a_json, &peer_b_json) {
        Ok(artifacts) => artifacts,
        Err(result) => {
            return LiveValidationResult::err(
                result
                    .errors
                    .into_iter()
                    .map(|e| LiveValidationError::ManifestVerification(format!("pairing: {e}")))
                    .collect(),
            )
        }
    };

    let compare_json_file =
        |filename: &str, expected_value: &Value, errors: &mut Vec<LiveValidationError>| {
            match read_required(filename) {
                Ok(json) => match serde_json::from_str::<Value>(&json) {
                    Ok(actual) => {
                        if !json_semantic_equal(&actual, expected_value) {
                            errors.push(LiveValidationError::ManifestVerification(format!(
                                "{filename}: semantic mismatch with recomputed artifact"
                            )));
                        }
                    }
                    Err(e) => errors.push(LiveValidationError::ManifestVerification(format!(
                        "{filename}: invalid JSON: {e}"
                    ))),
                },
                Err(err) => errors.push(err),
            }
        };

    compare_json_file("peer-a.summary.json", &expected.peer_a_summary, &mut errors);
    compare_json_file("peer-b.summary.json", &expected.peer_b_summary, &mut errors);

    let expected_pair = serde_json::to_value(&expected.pair).unwrap_or(Value::Null);
    compare_json_file("pair-summary.json", &expected_pair, &mut errors);

    match read_required("report.md") {
        Ok(actual_report) => {
            if actual_report != expected.report_markdown {
                errors.push(LiveValidationError::ManifestVerification(
                    "report.md: semantic mismatch with recomputed report".into(),
                ));
            }
        }
        Err(err) => errors.push(err),
    }

    let manifest_json = match read_required("artifact-manifest.json") {
        Ok(json) => json,
        Err(err) => return LiveValidationResult::err(vec![err]),
    };
    let manifest: Value = match serde_json::from_str(&manifest_json) {
        Ok(value) => value,
        Err(e) => {
            return LiveValidationResult::err(vec![LiveValidationError::ManifestVerification(
                e.to_string(),
            )])
        }
    };

    let identity_checks = [
        ("pairId", expected.pair.pair_id.clone()),
        (
            "sessionCorrelationId",
            expected.pair.session_correlation_id.clone(),
        ),
        ("softwareCommit", expected.pair.software_commit.clone()),
        ("schemaVersion", LIVE_MANIFEST_SCHEMA_VERSION.to_string()),
        ("algorithm", CHECKSUM_ALGORITHM.to_string()),
    ];
    for (field, expected_value) in identity_checks {
        let actual = manifest.get(field).and_then(|v| v.as_str());
        if actual != Some(expected_value.as_str()) {
            errors.push(LiveValidationError::ManifestVerification(format!(
                "artifact-manifest.json: {field} mismatch with recomputed pair"
            )));
        }
    }

    if errors.is_empty() {
        LiveValidationResult::ok()
    } else {
        LiveValidationResult::err(errors)
    }
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
    let checksum_result = verify_live_artifact_manifest(dir);
    if !checksum_result.valid {
        return checksum_result;
    }
    verify_live_directory_semantics(dir)
}
