use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliverySemantics {
    ReliableOrdered,
    ReliableUnordered,
    Deadline,
    LatestValue,
}
