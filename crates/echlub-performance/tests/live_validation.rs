use echlub_performance::{
    build_manifest_entries, compute_live_endpoint_derived, pair_live_endpoints,
    validate_cross_device_clock_timestamps, validate_live_endpoint, verify_live_artifact_manifest,
    LiveValidationError, LIVE_SCHEMA_VERSION,
};
use std::fs;
use std::path::PathBuf;

fn vector_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("test-vectors/performance")
        .join(name)
}

fn read_vector(name: &str) -> String {
    fs::read_to_string(vector_path(name)).unwrap()
}

#[test]
fn valid_live_endpoint_vector_passes() {
    let json = read_vector("live-endpoint-peer-a-v1.json");
    let result = validate_live_endpoint(&json, false);
    assert!(result.valid, "errors: {:?}", result.errors);
}

#[test]
fn finalized_positive_fixture_passes() {
    let json = read_vector("live-endpoint-finalized-peer-a-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(result.valid, "errors: {:?}", result.errors);
}

#[test]
fn cross_clock_plus_500_fixture_passes() {
    let json = read_vector("live-endpoint-cross-clock-plus-500-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(result.valid, "errors: {:?}", result.errors);
}

#[test]
fn cross_clock_minus_500_fixture_passes() {
    let json = read_vector("live-endpoint-cross-clock-minus-500-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(result.valid, "errors: {:?}", result.errors);
}

#[test]
fn cross_clock_rust_validator_accepts_separate_domains() {
    assert_eq!(
        validate_cross_device_clock_timestamps(1000.0, 1600.0, 1601.0, 1021.0),
        Some(20.0)
    );
    assert_eq!(
        validate_cross_device_clock_timestamps(2000.0, 1400.0, 1401.0, 2021.0),
        Some(20.0)
    );
    assert!(validate_cross_device_clock_timestamps(2000.0, 1500.0, 1501.0, 1000.0).is_none());
}

#[test]
fn rejects_missing_responder_fixture() {
    let json = read_vector("live-endpoint-missing-responder-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(!result.valid);
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e, LiveValidationError::InvalidResponderRole(_))));
}

#[test]
fn rejects_datachannel_closed_fixture() {
    let json = read_vector("live-endpoint-datachannel-closed-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(!result.valid);
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e, LiveValidationError::DataChannelNotOpen)));
}

#[test]
fn rejects_invalid_clock_only_fixture() {
    let json = read_vector("live-endpoint-invalid-clock-only-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(!result.valid);
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e, LiveValidationError::InsufficientClockProbes(_))));
}

#[test]
fn rejects_timestamp_reversal_fixture() {
    let json = read_vector("live-endpoint-timestamp-reversal-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(!result.valid);
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e, LiveValidationError::InvalidTimestamps)));
}

#[test]
fn rejects_draft_export_kind_as_finalized() {
    let json = read_vector("live-endpoint-draft-as-final-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(!result.valid);
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e, LiveValidationError::InvalidExportKind(_))));
}

#[test]
fn live_directory_manifest_verification_passes() {
    let dir = vector_path("live-directory-v1");
    let result = verify_live_artifact_manifest(&dir);
    assert!(result.valid, "errors: {:?}", result.errors);
}

#[test]
fn rejects_empty_manifest_fixture() {
    let dir = vector_path("live-directory-empty-manifest-v1");
    let result = verify_live_artifact_manifest(&dir);
    assert!(!result.valid);
}

#[test]
fn rejects_missing_manifest_entry_fixture() {
    let dir = vector_path("live-directory-missing-entry-v1");
    let result = verify_live_artifact_manifest(&dir);
    assert!(!result.valid);
}

#[test]
fn rejects_path_traversal_manifest_fixture() {
    let dir = vector_path("live-directory-path-traversal-v1");
    let result = verify_live_artifact_manifest(&dir);
    assert!(!result.valid);
}

#[test]
fn build_manifest_entries_fails_for_missing_file() {
    let temp = std::env::temp_dir().join("echlub-manifest-missing-file");
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).unwrap();
    let result = build_manifest_entries(
        &temp,
        "pair-test",
        "0123456789abcdef0123456789abcdef",
        "e4198657264a6b4629948469dcdabde21a3eaa34",
    );
    let _ = fs::remove_dir_all(&temp);
    assert!(result.is_err());
}

#[test]
fn live_directory_checksum_tamper_fails() {
    let src = vector_path("live-directory-v1");
    let temp = std::env::temp_dir().join("echlub-live-directory-tamper");
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).unwrap();
    for entry in fs::read_dir(&src).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), temp.join(entry.file_name())).unwrap();
    }

    let report_path = temp.join("report.md");
    let original = fs::read(&report_path).unwrap();
    let mut tampered = original.clone();
    if let Some(last) = tampered.last_mut() {
        *last ^= 0x01;
    }
    fs::write(&report_path, &tampered).unwrap();
    let result = verify_live_artifact_manifest(&temp);
    let _ = fs::remove_dir_all(&temp);
    assert!(!result.valid);
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e, LiveValidationError::ChecksumMismatch(_))));
}

#[test]
fn rejects_sdp_in_live_endpoint() {
    let json = format!(
        r#"{{"schemaVersion":"{LIVE_SCHEMA_VERSION}","evidenceLevel":"browser_network_observation","evidenceStatus":"exploratory_non_authoritative","peerRole":"peer_a","sessionCorrelationId":"0123456789abcdef0123456789abcdef","sdp":"v=0","limitations":["x"]}}"#
    );
    let result = validate_live_endpoint(&json, false);
    assert!(!result.valid);
}

#[test]
fn rejects_wrong_schema_version() {
    let json = r#"{"schemaVersion":"wrong","evidenceLevel":"browser_network_observation","evidenceStatus":"exploratory_non_authoritative","peerRole":"peer_a","sessionCorrelationId":"0123456789abcdef0123456789abcdef","limitations":["x"]}"#.to_string();
    let result = validate_live_endpoint(&json, false);
    assert!(matches!(
        result.errors.first(),
        Some(LiveValidationError::SchemaVersionMismatch(_))
    ));
}

#[test]
fn rejects_device_id_in_live_endpoint() {
    let json = format!(
        r#"{{"schemaVersion":"{LIVE_SCHEMA_VERSION}","evidenceLevel":"browser_network_observation","evidenceStatus":"exploratory_non_authoritative","peerRole":"peer_a","sessionCorrelationId":"0123456789abcdef0123456789abcdef","deviceId":"abc","limitations":["x"]}}"#
    );
    let result = validate_live_endpoint(&json, false);
    assert!(!result.valid);
}

#[test]
fn pair_rejects_same_role() {
    let json = read_vector("live-endpoint-peer-a-v1.json");
    let result = pair_live_endpoints(&json, &json);
    assert!(result.is_err());
}

#[test]
fn rejects_rtt_mismatch_fixture() {
    let json = read_vector("live-endpoint-rtt-mismatch-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(!result.valid);
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e, LiveValidationError::ClockRttMismatch)));
}

#[test]
fn rejects_offset_mismatch_fixture() {
    let json = read_vector("live-endpoint-offset-mismatch-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(!result.valid);
    assert!(result
        .errors
        .iter()
        .any(|e| matches!(e, LiveValidationError::ClockOffsetMismatch)));
}

#[test]
fn rejects_negative_stored_rtt_fixture() {
    let json = read_vector("live-endpoint-negative-stored-rtt-v1.json");
    let result = validate_live_endpoint(&json, true);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| {
        matches!(
            e,
            LiveValidationError::ClockRttMismatch | LiveValidationError::InvalidClockProbe
        )
    }));
}

#[test]
fn derived_clock_summary_excludes_invalid_probes() {
    let json = read_vector("live-endpoint-finalized-peer-a-v1.json");
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["clockProbes"]["samples"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "sequence": 999,
            "requesterRole": "peer_a",
            "responderRole": "peer_b",
            "protocolVersion": 1,
            "t0": 1000,
            "t1": 1010,
            "t2": 1011,
            "t3": 1021,
            "rttMs": 9999,
            "offsetMs": 0.0,
            "timeout": false,
            "duplicate": true,
            "unsolicited": false,
            "invalid": false
        }));
    let baseline: serde_json::Value = serde_json::from_str(&json).unwrap();
    let baseline_summary = compute_live_endpoint_derived(&baseline);
    let tampered_summary = compute_live_endpoint_derived(&value);
    assert_eq!(
        baseline_summary["medianClockProbeRttMs"],
        tampered_summary["medianClockProbeRttMs"]
    );
}

#[test]
fn pair_accepts_finalized_fixtures() {
    let a = read_vector("live-endpoint-finalized-peer-a-v1.json");
    let b = read_vector("live-endpoint-finalized-peer-b-v1.json");
    let result = pair_live_endpoints(&a, &b);
    assert!(result.is_ok(), "errors: {:?}", result.err());
}
