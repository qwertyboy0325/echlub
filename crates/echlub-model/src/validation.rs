use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationDecision {
    Accepted,
    Rejected,
    Duplicate,
    MissingDependency,
    StaleRevision,
    RequiresCoordination,
    UnsupportedVersion,
}
