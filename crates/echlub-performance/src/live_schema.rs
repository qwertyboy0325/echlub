use serde::{Deserialize, Serialize};

pub const LIVE_SCHEMA_VERSION: &str = "echlub.performance.live-observation/v1";
pub const LIVE_PAIR_SCHEMA_VERSION: &str = "echlub.performance.live-observation-pair/v1";
pub const LIVE_MANIFEST_SCHEMA_VERSION: &str = "echlub.performance.live-artifact-manifest/v1";
pub const CLOCK_PROBE_PROTOCOL_VERSION: i64 = 1;
pub const CHECKSUM_ALGORITHM: &str = "blake3";

pub const ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_A: &str = "live-endpoint-validated-peer-a";
pub const ARTIFACT_ROLE_ENDPOINT_VALIDATED_PEER_B: &str = "live-endpoint-validated-peer-b";
pub const ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_A: &str = "live-endpoint-summary-peer-a";
pub const ARTIFACT_ROLE_ENDPOINT_SUMMARY_PEER_B: &str = "live-endpoint-summary-peer-b";
pub const ARTIFACT_ROLE_PAIR_SUMMARY: &str = "live-observation-pair-summary";
pub const ARTIFACT_ROLE_PAIR_REPORT: &str = "live-pair-report";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LivePeerRole {
    PeerA,
    PeerB,
}

impl LivePeerRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            LivePeerRole::PeerA => "peer_a",
            LivePeerRole::PeerB => "peer_b",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiveExportKind {
    Finalized,
    DiagnosticDraft,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LiveMetricValue {
    ObservedNumber { value: f64 },
    ObservedBoolean { value: bool },
    ObservedCategory { value: String },
    Unsupported,
    Unavailable { reason: String },
    Invalid { reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveClockProbeSampleV1 {
    pub sequence: u64,
    #[serde(alias = "senderRole")]
    pub requester_role: LivePeerRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub responder_role: Option<LivePeerRole>,
    pub protocol_version: i64,
    pub t0: f64,
    pub t1: Option<f64>,
    pub t2: Option<f64>,
    pub t3: Option<f64>,
    pub rtt_ms: Option<f64>,
    pub offset_ms: Option<f64>,
    pub timeout: bool,
    pub duplicate: bool,
    pub unsolicited: bool,
    pub invalid: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveClockProbesV1 {
    pub samples: Vec<LiveClockProbeSampleV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_probes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median_rtt_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub median_offset_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mad_rtt_ms: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub offset_limitation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveConnectionLifecycleV1 {
    pub peer_connection_state: String,
    pub ice_connection_state: String,
    pub signaling_state: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveDataChannelV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ordered: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_retransmits: Option<u64>,
    pub ready_state: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LivePlayoutV1 {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub remote_audio_track_received: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_audio_track_ready_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_audio_track_unmuted: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub autoplay_attempted: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStatsSampleV1 {
    pub offset_ms: f64,
    pub inbound_audio: serde_json::Value,
    pub outbound_audio: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_pair: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_inbound_audio: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub codec: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval_metrics: Option<serde_json::Value>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub export_kind: Option<LiveExportKind>,
    pub environment: serde_json::Value,
    pub capture: serde_json::Value,
    pub playout: LivePlayoutV1,
    pub connection_lifecycle: LiveConnectionLifecycleV1,
    pub data_channel: LiveDataChannelV1,
    pub clock_probes: LiveClockProbesV1,
    pub stats_samples: Vec<LiveStatsSampleV1>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_checksums: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveArtifactManifestEntryV1 {
    pub filename: String,
    pub checksum: String,
    pub artifact_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveArtifactManifestV1 {
    pub schema_version: String,
    pub algorithm: String,
    pub pair_id: String,
    pub session_correlation_id: String,
    pub software_commit: String,
    pub artifacts: Vec<LiveArtifactManifestEntryV1>,
}
