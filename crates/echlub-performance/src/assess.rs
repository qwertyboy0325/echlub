use crate::schema::{EvidenceLevel, MetricValue, PerformanceRunV1};
use crate::validate::{parse_and_validate, ValidationError, ValidationResult};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum AssessmentError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("no pulse detections")]
    NoPulseDetections,
    #[error("insufficient pulses emitted")]
    InsufficientPulsesEmitted,
    #[error("insufficient pulses detected")]
    InsufficientPulsesDetected,
    #[error("zero bytes received on inbound RTP")]
    ZeroBytesReceived,
    #[error("detection rate below minimum threshold")]
    DetectionRateBelowThreshold,
    #[error("detected count exceeds emitted count")]
    DetectedExceedsEmitted,
    #[error("detection rate inconsistent with counts")]
    InconsistentDetectionRate,
    #[error("non-positive observation window")]
    NonPositiveObservationWindow,
    #[error("invalid latency semantics")]
    InvalidLatencySemantics,
    #[error("detect before emit")]
    DetectBeforeEmit,
    #[error("suspicious zero observed metric: {0}")]
    SuspiciousZeroObserved(String),
}

#[derive(Debug, PartialEq)]
pub struct AssessmentResult {
    pub pass: bool,
    pub errors: Vec<AssessmentError>,
}

impl AssessmentResult {
    pub fn ok() -> Self {
        Self {
            pass: true,
            errors: vec![],
        }
    }

    pub fn err(errors: Vec<AssessmentError>) -> Self {
        Self {
            pass: false,
            errors,
        }
    }
}

const MIN_PULSES_EMITTED: f64 = 5.0;
const MIN_PULSES_DETECTED: f64 = 4.0;
const MIN_DETECTION_RATE: f64 = 0.8;

pub fn assess_synthetic_observation(json: &str) -> AssessmentResult {
    let run = match parse_and_validate(json) {
        Ok(run) => run,
        Err(ValidationResult { errors, .. }) => {
            return AssessmentResult::err(vec![AssessmentError::Validation(
                format_validation_errors(&errors),
            )]);
        }
    };

    assess_run(&run)
}

pub fn assess_run(run: &PerformanceRunV1) -> AssessmentResult {
    if run.evidence_level != EvidenceLevel::BrowserSyntheticMediaPath {
        return AssessmentResult::ok();
    }

    let mut errors = Vec::new();

    let pulses_emitted = observed_value(&run.synthetic_pulse.pulses_emitted);
    let pulses_detected = observed_value(&run.synthetic_pulse.pulses_detected);
    let detection_rate = observed_value(&run.synthetic_pulse.detection_rate);
    let bytes_received = observed_value(&run.transport.bytes_received);

    if pulses_emitted
        .map(|v| v < MIN_PULSES_EMITTED)
        .unwrap_or(true)
    {
        errors.push(AssessmentError::InsufficientPulsesEmitted);
    }

    if pulses_detected
        .map(|v| v < MIN_PULSES_DETECTED)
        .unwrap_or(true)
    {
        errors.push(AssessmentError::InsufficientPulsesDetected);
    }

    if detection_rate
        .map(|v| v < MIN_DETECTION_RATE)
        .unwrap_or(true)
    {
        errors.push(AssessmentError::DetectionRateBelowThreshold);
    }

    if let (Some(emitted), Some(detected)) = (pulses_emitted, pulses_detected) {
        if detected > emitted {
            errors.push(AssessmentError::DetectedExceedsEmitted);
        }
        if emitted > 0.0 {
            let expected = detected / emitted;
            if detection_rate
                .map(|r| (r - expected).abs() > 0.01)
                .unwrap_or(true)
            {
                errors.push(AssessmentError::InconsistentDetectionRate);
            }
        }
    }

    if bytes_received.map(|v| v <= 0.0).unwrap_or(true) {
        errors.push(AssessmentError::ZeroBytesReceived);
    }

    if let (Some(start), Some(end)) = (
        run.metadata
            .started_at
            .parse::<chrono::DateTime<chrono::Utc>>()
            .ok(),
        run.metadata
            .completed_at
            .parse::<chrono::DateTime<chrono::Utc>>()
            .ok(),
    ) {
        if end <= start {
            errors.push(AssessmentError::NonPositiveObservationWindow);
        }
    }

    let emit = observed_value(&run.timing.pulse_emit_ms);
    let detect = observed_value(&run.timing.pulse_detect_ms);
    let loopback = observed_value(&run.timing.loopback_latency_ms);

    if let (Some(e), Some(d)) = (emit, detect) {
        if d < e {
            errors.push(AssessmentError::DetectBeforeEmit);
        }
    }

    if let Some(l) = loopback {
        if !l.is_finite() || l < 0.0 {
            errors.push(AssessmentError::InvalidLatencySemantics);
        }
    }

    if pulses_detected
        .map(|v| v < MIN_PULSES_DETECTED)
        .unwrap_or(true)
    {
        check_suspicious_zeros(run, &mut errors);
    }

    if errors.is_empty() {
        AssessmentResult::ok()
    } else {
        AssessmentResult::err(errors)
    }
}

fn observed_value(metric: &MetricValue) -> Option<f64> {
    match metric {
        MetricValue::Observed { value } => Some(*value),
        _ => None,
    }
}

fn check_suspicious_zeros(run: &PerformanceRunV1, errors: &mut Vec<AssessmentError>) {
    let suspicious = [
        ("timing.pulseDetectMs", &run.timing.pulse_detect_ms),
        ("timing.loopbackLatencyMs", &run.timing.loopback_latency_ms),
        (
            "syntheticPulse.meanDetectionLatencyMs",
            &run.synthetic_pulse.mean_detection_latency_ms,
        ),
        ("syntheticPulse.jitterMs", &run.synthetic_pulse.jitter_ms),
    ];
    for (path, metric) in suspicious {
        if matches!(metric, MetricValue::Observed { value } if *value == 0.0) {
            errors.push(AssessmentError::SuspiciousZeroObserved(path.to_string()));
        }
    }
}

fn format_validation_errors(errors: &[ValidationError]) -> String {
    errors
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}
