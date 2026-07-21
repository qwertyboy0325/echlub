use serde::{Deserialize, Serialize};

pub const LIVE_SCHEMA_VERSION: &str = "echlub.performance.live-observation/v1";
pub const LIVE_PAIR_SCHEMA_VERSION: &str = "echlub.performance.live-observation-pair/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LivePeerRole {
    PeerA,
    PeerB,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LiveMetricValue {
    Observed { value: f64 },
    Unsupported,
    Unavailable { reason: String },
    Invalid { reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveEndpointObservationV1 {
    pub schema_version: String,
    pub evidence_level: String,
    pub evidence_status: String,
    pub run_id: String,
    pub session_correlation_id: String,
    pub peer_role: LivePeerRole,
    pub started_at_utc: String,
    pub completed_at_utc: String,
    pub software_commit: String,
    pub environment: serde_json::Value,
    pub capture: serde_json::Value,
    pub playout: serde_json::Value,
    pub connection_lifecycle: serde_json::Value,
    pub data_channel: serde_json::Value,
    pub clock_probes: serde_json::Value,
    pub stats_samples: serde_json::Value,
    pub unsupported_metrics: serde_json::Value,
    pub limitations: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derived: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveObservationPairV1 {
    pub schema_version: String,
    pub pair_id: String,
    pub session_correlation_id: String,
    pub software_commit: String,
    pub endpoint_run_ids: Vec<String>,
    pub observation_overlap_seconds: f64,
    pub per_endpoint_summaries: serde_json::Value,
    pub consistency_checks: serde_json::Value,
    pub limitations: Vec<String>,
    pub artifact_checksums: serde_json::Value,
}
