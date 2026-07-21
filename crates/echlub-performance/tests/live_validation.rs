use echlub_performance::{
    pair_live_endpoints, validate_live_endpoint, LiveValidationError, LIVE_SCHEMA_VERSION,
};
use std::fs;
use std::path::PathBuf;

fn vector_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("test-vectors/performance")
        .join(name)
}

#[test]
fn valid_live_endpoint_vector_passes() {
    let json = fs::read_to_string(vector_path("live-endpoint-peer-a-v1.json")).unwrap();
    let result = validate_live_endpoint(&json, false);
    assert!(result.valid, "errors: {:?}", result.errors);
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
    let json = fs::read_to_string(vector_path("live-endpoint-peer-a-v1.json")).unwrap();
    let result = pair_live_endpoints(&json, &json);
    assert!(result.is_err());
}
