use echlub_model::{EditIntent, OperationEnvelope, StateHash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtocolFrame {
    OperationBatch(OperationBatch),
    IntentUpdate(IntentUpdate),
    ValidationReceipt(ValidationReceipt),
    Checkpoint(CheckpointFrame),
    MissingOperationsRequest(MissingOperationsRequest),
    CapabilityHello(CapabilityHello),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationBatch {
    pub protocol_version: u32,
    pub operations: Vec<OperationEnvelope>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentUpdate {
    pub intent: EditIntent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationReceipt {
    pub operation_id: echlub_model::OperationId,
    pub decision: echlub_model::ValidationDecision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointFrame {
    pub state_hash: StateHash,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingOperationsRequest {
    pub from_actor_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityHello {
    pub protocol_version: u32,
    pub supported_flows: Vec<super::flow::Flow>,
}
