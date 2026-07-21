use echlub_model::{DocumentId, SessionId};
use echlub_protocol::flow::Flow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEpoch {
    pub session_id: SessionId,
    pub epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityContract {
    pub protocol_version: u32,
    pub document_id: DocumentId,
    pub supported_flows: Vec<Flow>,
    pub live_audio_transport_implemented: bool,
    pub persistence_implemented: bool,
    pub authentication_implemented: bool,
    pub signaling_implemented: bool,
}

impl CapabilityContract {
    pub fn foundation_baseline() -> Self {
        Self {
            protocol_version: 1,
            document_id: DocumentId(1),
            supported_flows: vec![
                Flow::Control,
                Flow::Operations,
                Flow::Intent,
                Flow::Receipts,
                Flow::Checkpoint,
                Flow::Clock,
                Flow::PerformanceAudio,
                Flow::Assets,
            ],
            live_audio_transport_implemented: false,
            persistence_implemented: false,
            authentication_implemented: false,
            signaling_implemented: false,
        }
    }
}
