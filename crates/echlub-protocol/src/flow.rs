use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Flow {
    Control,
    Operations,
    Intent,
    Receipts,
    Checkpoint,
    Clock,
    PerformanceAudio,
    Assets,
}

impl Flow {
    pub fn required_delivery(self) -> super::delivery::DeliverySemantics {
        use super::delivery::DeliverySemantics::*;
        use Flow::*;
        match self {
            Control | Checkpoint | Assets => ReliableOrdered,
            Operations | Receipts => ReliableUnordered,
            Intent | Clock => LatestValue,
            PerformanceAudio => Deadline,
        }
    }
}
