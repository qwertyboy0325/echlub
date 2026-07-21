use serde::{Deserialize, Serialize};

use crate::ids::{ActorId, DocumentId, OperationId};
use crate::operation::Operation;

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationEnvelope {
    pub protocol_version: u32,
    pub document_id: DocumentId,
    pub operation_id: OperationId,
    pub actor_id: ActorId,
    pub actor_sequence: u64,
    pub operation: Operation,
}
