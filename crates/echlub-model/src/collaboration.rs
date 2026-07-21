use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationRelation {
    Independent,
    Commutative,
    Mergeable,
    RequiresCoordination,
    DestructiveConflict,
}
