use echlub_performance::{
    build_manifest_entries, compute_live_endpoint_derived, pair_live_endpoints,
    validate_cross_device_clock_timestamps, validate_live_directory, validate_live_endpoint,
    verify_live_artifact_manifest, LiveValidationError, LIVE_SCHEMA_VERSION,
};
use std::fs;
use std::path::{Path, PathBuf};

fn vector_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("test-vectors/performance")
        .join(name)
}

fn read_vector(name: &str) -> String {
    fs::read_to_string(vector_path(name)).unwrap()
}

fn copy_directory_fixture(name: &str, temp_name: &str) -> PathBuf {
    let src = vector_path(name);
    let temp = std::env::temp_dir().join(temp_name);
    let _ = fs::remove_dir_all(&temp);
    fs::create_dir_all(&temp).unwrap();
    for entry in fs::read_dir(&src).unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), temp.join(entry.file_name())).unwrap();
    }
    temp
}

fn rehash_manifest_file(temp: &Path, filename: &str, bytes: &[u8]) {
    use echlub_performance::checksum_bytes;
    let manifest_path = temp.join("artifact-manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    for entry in manifest["artifacts"].as_array_mut().unwrap() {
        if entry["filename"] == filename {
            entry["checksum"] = serde_json::json!(checksum_bytes(bytes));
        }
    }
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn assert_directory_semantic_failure(temp: &Path, needle: &str) {
    let result = validate_live_directory(temp);
    assert!(!result.valid, "expected semantic failure for {needle}");
    assert!(result.errors.iter().any(|e| {
        matches!(
            e,
            LiveValidationError::ManifestVerification(msg) if msg.contains(needle)
        )
    }));
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
    let result = validate_live_directory(&dir);
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
fn rejects_transport_only_packet_progression_fixture() {
    let mut json: serde_json::Value =
        serde_json::from_str(&read_vector("live-endpoint-finalized-peer-a-v1.json")).unwrap();
    for sample in json["statsSamples"].as_array_mut().unwrap() {
        sample["inboundAudio"] = serde_json::json!({});
        sample["outboundAudio"] = serde_json::json!({});
        if let Some(cp) = sample.get_mut("candidatePair") {
            cp["packetsReceived"] = serde_json::json!({"kind":"observed_number","value":100});
            cp["packetsSent"] = serde_json::json!({"kind":"observed_number","value":100});
        }
    }
    let result = validate_live_endpoint(&json.to_string(), true);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| {
        matches!(
            e,
            LiveValidationError::NoInboundRtpAudioProgression
                | LiveValidationError::NoOutboundRtpAudioProgression
                | LiveValidationError::InvalidAudioCounterProvenance
        )
    }));
}

#[test]
fn live_directory_semantic_summary_mismatch_fails_after_checksum_regeneration() {
    let temp = copy_directory_fixture(
        "live-directory-v1",
        "echlub-live-directory-semantic-mismatch",
    );
    let summary_path = temp.join("peer-a.summary.json");
    let mut summary: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&summary_path).unwrap()).unwrap();
    if let Some(obj) = summary.as_object_mut() {
        obj.insert("staleMarker".into(), serde_json::json!("semantic-tamper"));
    }
    let summary_bytes = serde_json::to_vec_pretty(&summary).unwrap();
    fs::write(&summary_path, &summary_bytes).unwrap();
    rehash_manifest_file(&temp, "peer-a.summary.json", &summary_bytes);
    assert_directory_semantic_failure(&temp, "peer-a.summary.json");
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn live_directory_semantic_peer_b_summary_mismatch_fails() {
    let temp = copy_directory_fixture("live-directory-v1", "echlub-live-directory-peer-b-summary");
    let path = temp.join("peer-b.summary.json");
    let mut summary: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    summary["medianClockProbeRttMs"] = serde_json::json!(9999);
    let bytes = serde_json::to_vec_pretty(&summary).unwrap();
    fs::write(&path, &bytes).unwrap();
    rehash_manifest_file(&temp, "peer-b.summary.json", &bytes);
    assert_directory_semantic_failure(&temp, "peer-b.summary.json");
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn live_directory_semantic_pair_summary_correlation_mismatch_fails() {
    let temp = copy_directory_fixture("live-directory-v1", "echlub-live-directory-pair-corr");
    let path = temp.join("pair-summary.json");
    let mut pair: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    pair["sessionCorrelationId"] = serde_json::json!("0123456789abcdef0123456789abcde0");
    let bytes = serde_json::to_vec_pretty(&pair).unwrap();
    fs::write(&path, &bytes).unwrap();
    rehash_manifest_file(&temp, "pair-summary.json", &bytes);
    assert_directory_semantic_failure(&temp, "pair-summary.json");
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn live_directory_semantic_pair_summary_run_ids_mismatch_fails() {
    let temp = copy_directory_fixture("live-directory-v1", "echlub-live-directory-pair-run-ids");
    let path = temp.join("pair-summary.json");
    let mut pair: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    pair["endpointRunIds"] = serde_json::json!(["stale-a", "stale-b"]);
    let bytes = serde_json::to_vec_pretty(&pair).unwrap();
    fs::write(&path, &bytes).unwrap();
    rehash_manifest_file(&temp, "pair-summary.json", &bytes);
    assert_directory_semantic_failure(&temp, "pair-summary.json");
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn live_directory_semantic_stale_report_fails_after_checksum_regeneration() {
    let temp = copy_directory_fixture("live-directory-v1", "echlub-live-directory-stale-report");
    let path = temp.join("report.md");
    let mut report = fs::read_to_string(&path).unwrap();
    report.push_str("\nstale appendix\n");
    fs::write(&path, &report).unwrap();
    rehash_manifest_file(&temp, "report.md", report.as_bytes());
    assert_directory_semantic_failure(&temp, "report.md");
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn live_directory_semantic_manifest_pair_id_mismatch_fails() {
    let temp = copy_directory_fixture(
        "live-directory-v1",
        "echlub-live-directory-manifest-pair-id",
    );
    let manifest_path = temp.join("artifact-manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["pairId"] = serde_json::json!("pair-deadbeefdeadbeef");
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    assert_directory_semantic_failure(&temp, "pairId");
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn live_directory_semantic_manifest_commit_mismatch_fails() {
    let temp = copy_directory_fixture("live-directory-v1", "echlub-live-directory-manifest-commit");
    let manifest_path = temp.join("artifact-manifest.json");
    let mut manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest_path).unwrap()).unwrap();
    manifest["softwareCommit"] = serde_json::json!("0000000000000000000000000000000000000000");
    fs::write(
        manifest_path,
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    assert_directory_semantic_failure(&temp, "softwareCommit");
    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn playout_remote_audio_track_unmuted_round_trips_through_rust_schema() {
    use echlub_performance::live_schema::LivePlayoutV1;
    let playout_json = serde_json::json!({
      "remoteAudioTrackReceived": true,
      "remoteAudioTrackReadyState": "live",
      "remoteAudioTrackUnmuted": true,
      "autoplayAttempted": true
    });
    let playout: LivePlayoutV1 = serde_json::from_value(playout_json.clone()).unwrap();
    assert_eq!(playout.remote_audio_track_unmuted, Some(true));
    assert_eq!(
        playout_json["remoteAudioTrackUnmuted"],
        serde_json::json!(true)
    );
    assert!(playout_json.get("remoteAudioTrackMuted").is_none());
    let reserialized = serde_json::to_value(&playout).unwrap();
    assert_eq!(
        reserialized["remoteAudioTrackUnmuted"],
        serde_json::json!(true)
    );
    assert!(reserialized.get("remoteAudioTrackMuted").is_none());
}

#[test]
fn pair_accepts_finalized_fixtures() {
    let a = read_vector("live-endpoint-finalized-peer-a-v1.json");
    let b = read_vector("live-endpoint-finalized-peer-b-v1.json");
    let result = pair_live_endpoints(&a, &b);
    assert!(result.is_ok(), "errors: {:?}", result.err());
}
