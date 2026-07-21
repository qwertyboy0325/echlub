use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "PerformanceRunV1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLevel {
    HarnessValidation,
    BrowserSyntheticMediaPath,
    BrowserNetworkObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    ExploratoryNonAuthoritative,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MetricValue {
    Observed { value: f64 },
    Unsupported,
    Unavailable { reason: String },
    Invalid { reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunMetadata {
    pub run_id: String,
    pub started_at: String,
    pub completed_at: String,
    pub harness_version: String,
    pub browser_family: Option<String>,
    pub platform: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimingMetrics {
    pub pulse_emit_ms: MetricValue,
    pub pulse_detect_ms: MetricValue,
    pub loopback_latency_ms: MetricValue,
    pub datachannel_rtt_ms: MetricValue,
    pub ice_gathering_ms: MetricValue,
    pub connection_setup_ms: MetricValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyntheticPulseMetrics {
    pub pulses_emitted: MetricValue,
    pub pulses_detected: MetricValue,
    pub detection_rate: MetricValue,
    pub mean_detection_latency_ms: MetricValue,
    pub jitter_ms: MetricValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportObservation {
    pub candidate_pair_type: MetricValue,
    pub bytes_sent: MetricValue,
    pub bytes_received: MetricValue,
    pub packets_lost: MetricValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedMetrics {
    pub end_to_end_synthetic_ms: MetricValue,
    pub setup_to_first_pulse_ms: MetricValue,
    pub datachannel_overhead_ms: MetricValue,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceRunV1 {
    pub schema_version: String,
    pub evidence_level: EvidenceLevel,
    pub evidence_status: EvidenceStatus,
    pub metadata: RunMetadata,
    pub timing: TimingMetrics,
    pub synthetic_pulse: SyntheticPulseMetrics,
    pub transport: TransportObservation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived: Option<DerivedMetrics>,
}

impl PerformanceRunV1 {
    pub fn with_derived(mut self) -> Self {
        self.derived = Some(crate::derived::compute_derived_metrics(&self));
        self
    }
}
