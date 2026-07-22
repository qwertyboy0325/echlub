use echlub_performance::{
    pair_live_endpoints, validate_live_endpoint, verify_live_artifact_manifest,
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
fn pair_accepts_finalized_fixtures() {
    let a = read_vector("live-endpoint-finalized-peer-a-v1.json");
    let b = read_vector("live-endpoint-finalized-peer-b-v1.json");
    let result = pair_live_endpoints(&a, &b);
    assert!(result.is_ok(), "errors: {:?}", result.err());
}
