use serde::{Deserialize, Serialize};

use crate::ids::{ActorId, NoteId, TrackId};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditIntent {
    pub actor_id: ActorId,
    pub target: EditTarget,
    pub action: EditAction,
    pub expiry: IntentExpiry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditTarget {
    Track { track_id: TrackId },
    Note { note_id: NoteId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditAction {
    Create,
    Move,
    Delete,
    Rename,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntentExpiry {
    pub sequence: u64,
}
