use echlub_performance::{
    compute_derived_metrics, summarize_run, validate_run, EvidenceLevel, EvidenceStatus,
    MetricValue, PerformanceRunV1, RunMetadata, SyntheticPulseMetrics, TimingMetrics,
    TransportObservation, SCHEMA_VERSION,
};

fn sample_run() -> PerformanceRunV1 {
    PerformanceRunV1 {
        schema_version: SCHEMA_VERSION.to_string(),
        evidence_level: EvidenceLevel::BrowserSyntheticMediaPath,
        evidence_status: EvidenceStatus::ExploratoryNonAuthoritative,
        metadata: RunMetadata {
            run_id: "test-run-001".into(),
            started_at: "2026-07-21T08:00:00Z".into(),
            completed_at: "2026-07-21T08:00:05Z".into(),
            harness_version: "0.1.0".into(),
            browser_family: Some("chromium".into()),
            platform: Some("darwin".into()),
        },
        timing: TimingMetrics {
            pulse_emit_ms: MetricValue::Observed { value: 1.5 },
            pulse_detect_ms: MetricValue::Observed { value: 12.3 },
            loopback_latency_ms: MetricValue::Observed { value: 45.0 },
            datachannel_rtt_ms: MetricValue::Observed { value: 2.1 },
            ice_gathering_ms: MetricValue::Observed { value: 120.0 },
            connection_setup_ms: MetricValue::Observed { value: 350.0 },
        },
        synthetic_pulse: SyntheticPulseMetrics {
            pulses_emitted: MetricValue::Observed { value: 10.0 },
            pulses_detected: MetricValue::Observed { value: 10.0 },
            detection_rate: MetricValue::Observed { value: 1.0 },
            mean_detection_latency_ms: MetricValue::Observed { value: 45.0 },
            jitter_ms: MetricValue::Observed { value: 3.2 },
        },
        transport: TransportObservation {
            candidate_pair_type: MetricValue::Observed { value: 1.0 },
            bytes_sent: MetricValue::Observed { value: 4096.0 },
            bytes_received: MetricValue::Observed { value: 4096.0 },
            packets_lost: MetricValue::Observed { value: 0.0 },
        },
        derived: None,
    }
}

#[test]
fn valid_run_passes_validation() {
    let run = sample_run();
    let json = serde_json::to_string(&run).unwrap();
    let result = validate_run(&json);
    assert!(result.valid, "errors: {:?}", result.errors);
}

#[test]
fn rejects_physical_acoustic_evidence_level() {
    let json = r#"{"evidenceLevel":"physical_acoustic_mouth_to_ear","evidenceStatus":"exploratory_non_authoritative","schemaVersion":"PerformanceRunV1"}"#;
    let result = validate_run(json);
    assert!(!result.valid);
}

#[test]
fn rejects_forbidden_sdp_content() {
    let json = r#"{"evidenceLevel":"browser_synthetic_media_path","evidenceStatus":"exploratory_non_authoritative","schemaVersion":"PerformanceRunV1","sdp":"v=0 candidate:123"}"#;
    let result = validate_run(json);
    assert!(!result.valid);
}

#[test]
fn rejects_forbidden_device_id_key() {
    let json = r#"{"evidenceLevel":"browser_synthetic_media_path","evidenceStatus":"exploratory_non_authoritative","schemaVersion":"PerformanceRunV1","metadata":{"deviceId":"abc"}}"#;
    let result = validate_run(json);
    assert!(!result.valid);
}

#[test]
fn rejects_wrong_schema_version() {
    let mut run = sample_run();
    run.schema_version = "PerformanceRunV0".into();
    let json = serde_json::to_string(&run).unwrap();
    let result = validate_run(&json);
    assert!(!result.valid);
}

#[test]
fn rejects_wrong_evidence_status() {
    let json = r#"{"evidenceLevel":"browser_synthetic_media_path","evidenceStatus":"authoritative","schemaVersion":"PerformanceRunV1"}"#;
    let result = validate_run(json);
    assert!(!result.valid);
}

#[test]
fn derived_metrics_computed_correctly() {
    let run = sample_run();
    let derived = compute_derived_metrics(&run);
    match derived.end_to_end_synthetic_ms {
        MetricValue::Observed { value } => {
            assert!((value - 55.8).abs() < 0.01);
        }
        other => panic!("expected observed, got {other:?}"),
    }
    match derived.setup_to_first_pulse_ms {
        MetricValue::Observed { value } => {
            assert!((value - 351.5).abs() < 0.01);
        }
        other => panic!("expected observed, got {other:?}"),
    }
}

#[test]
fn derived_metrics_unavailable_when_missing() {
    let mut run = sample_run();
    run.timing.loopback_latency_ms = MetricValue::Unavailable {
        reason: "no loopback".into(),
    };
    run.timing.pulse_emit_ms = MetricValue::Unavailable {
        reason: "no emit".into(),
    };
    run.timing.pulse_detect_ms = MetricValue::Unavailable {
        reason: "no detect".into(),
    };
    run.timing.connection_setup_ms = MetricValue::Unavailable {
        reason: "no setup".into(),
    };
    let derived = compute_derived_metrics(&run);
    assert!(matches!(
        derived.end_to_end_synthetic_ms,
        MetricValue::Unavailable { .. }
    ));
}

#[test]
fn summarize_includes_disclaimer() {
    let run = sample_run().with_derived();
    let report = summarize_run(&run);
    assert!(report.markdown.contains("Exploratory non-authoritative"));
    assert!(report.markdown.contains("Not an acoustic mouth-to-ear"));
    assert!(report.markdown.contains("Not a transport selection"));
}

#[test]
fn metric_value_variants_serialize() {
    let observed = MetricValue::Observed { value: 42.0 };
    let json = serde_json::to_string(&observed).unwrap();
    assert!(json.contains("\"kind\":\"observed\""));

    let unsupported = MetricValue::Unsupported;
    let json = serde_json::to_string(&unsupported).unwrap();
    assert!(json.contains("\"kind\":\"unsupported\""));

    let unavailable = MetricValue::Unavailable {
        reason: "test".into(),
    };
    let json = serde_json::to_string(&unavailable).unwrap();
    assert!(json.contains("\"kind\":\"unavailable\""));
}
