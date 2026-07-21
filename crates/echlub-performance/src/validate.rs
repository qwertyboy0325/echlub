use crate::privacy::{contains_forbidden_content, scan_value_for_forbidden_keys};
use crate::schema::{EvidenceLevel, EvidenceStatus, MetricValue, PerformanceRunV1, SCHEMA_VERSION};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    #[error("invalid JSON: {0}")]
    InvalidJson(String),
    #[error("schema version mismatch: expected {SCHEMA_VERSION}, got {0}")]
    SchemaVersionMismatch(String),
    #[error("forbidden evidence level: PhysicalAcousticMouthToEar")]
    ForbiddenEvidenceLevel,
    #[error("invalid evidence level: {0}")]
    InvalidEvidenceLevel(String),
    #[error("evidence status must be exploratory_non_authoritative")]
    InvalidEvidenceStatus,
    #[error("privacy violation: {0}")]
    PrivacyViolation(String),
    #[error("missing required field: {0}")]
    MissingField(String),
    #[error("metric zero-filled where unavailable expected: {0}")]
    ZeroFilledMetric(String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
}

impl ValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
        }
    }

    pub fn err(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            errors,
        }
    }
}

pub fn validate_run(json: &str) -> ValidationResult {
    let mut errors = Vec::new();

    if let Some(violation) = contains_forbidden_content(json) {
        errors.push(ValidationError::PrivacyViolation(violation));
    }

    let value: Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(e) => return ValidationResult::err(vec![ValidationError::InvalidJson(e.to_string())]),
    };

    if let Some(violation) = scan_value_for_forbidden_keys(&value, "") {
        errors.push(ValidationError::PrivacyViolation(violation));
    }

    let raw_level = value
        .get("evidenceLevel")
        .or_else(|| value.get("evidence_level"))
        .and_then(|v| v.as_str());

    if let Some(level) = raw_level {
        if level == "physical_acoustic_mouth_to_ear" || level == "PhysicalAcousticMouthToEar" {
            errors.push(ValidationError::ForbiddenEvidenceLevel);
        }
        if !matches!(
            level,
            "harness_validation"
                | "browser_synthetic_media_path"
                | "browser_network_observation"
                | "HarnessValidation"
                | "BrowserSyntheticMediaPath"
                | "BrowserNetworkObservation"
        ) {
            errors.push(ValidationError::InvalidEvidenceLevel(level.to_string()));
        }
    } else {
        errors.push(ValidationError::MissingField("evidenceLevel".into()));
    }

    let raw_status = value
        .get("evidenceStatus")
        .or_else(|| value.get("evidence_status"))
        .and_then(|v| v.as_str());

    match raw_status {
        Some("exploratory_non_authoritative") | Some("ExploratoryNonAuthoritative") => {}
        _ => errors.push(ValidationError::InvalidEvidenceStatus),
    }

    let schema_version = value
        .get("schemaVersion")
        .or_else(|| value.get("schema_version"))
        .and_then(|v| v.as_str());

    match schema_version {
        Some(v) if v == SCHEMA_VERSION => {}
        Some(v) => errors.push(ValidationError::SchemaVersionMismatch(v.to_string())),
        None => errors.push(ValidationError::MissingField("schemaVersion".into())),
    }

    check_zero_filled_metrics(&value, &mut errors);

    if errors.is_empty() {
        ValidationResult::ok()
    } else {
        ValidationResult::err(errors)
    }
}

fn check_zero_filled_metrics(value: &Value, errors: &mut Vec<ValidationError>) {
    fn walk(val: &Value, path: &str, errors: &mut Vec<ValidationError>) {
        if let Some(obj) = val.as_object() {
            if obj.get("kind").and_then(|k| k.as_str()) == Some("observed") {
                if let Some(v) = obj.get("value").and_then(|v| v.as_f64()) {
                    if v == 0.0 && path.contains("Unavailable") {
                        errors.push(ValidationError::ZeroFilledMetric(path.to_string()));
                    }
                }
            }
            for (key, child) in obj {
                walk(child, &format!("{path}.{key}"), errors);
            }
        } else if let Some(arr) = val.as_array() {
            for (i, child) in arr.iter().enumerate() {
                walk(child, &format!("{path}[{i}]"), errors);
            }
        }
    }
    walk(value, "", errors);
}

pub fn parse_and_validate(json: &str) -> Result<PerformanceRunV1, ValidationResult> {
    let result = validate_run(json);
    if !result.valid {
        return Err(result);
    }
    let run: PerformanceRunV1 = serde_json::from_str(json)
        .map_err(|e| ValidationResult::err(vec![ValidationError::InvalidJson(e.to_string())]))?;

    if run.evidence_status != EvidenceStatus::ExploratoryNonAuthoritative {
        return Err(ValidationResult::err(vec![
            ValidationError::InvalidEvidenceStatus,
        ]));
    }

    Ok(run)
}

pub fn metric_is_observed(metric: &MetricValue) -> bool {
    matches!(metric, MetricValue::Observed { .. })
}

pub fn allowed_evidence_level(level: &EvidenceLevel) -> bool {
    matches!(
        level,
        EvidenceLevel::HarnessValidation
            | EvidenceLevel::BrowserSyntheticMediaPath
            | EvidenceLevel::BrowserNetworkObservation
    )
}
