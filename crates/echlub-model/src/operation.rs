use serde::{Deserialize, Serialize};

use crate::document::DocumentRevision;
use crate::ids::{NoteId, TrackId};
use crate::time::Tick;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    AddTrack(AddTrack),
    AddNote(AddNote),
    MoveNote(MoveNote),
    DeleteNote(DeleteNote),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddTrack {
    pub track_id: TrackId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddNote {
    pub track_id: TrackId,
    pub note_id: NoteId,
    pub start: Tick,
    pub duration: Tick,
    pub pitch: u8,
    pub velocity: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveNote {
    pub note_id: NoteId,
    pub expected_revision: DocumentRevision,
    pub new_start: Tick,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeleteNote {
    pub note_id: NoteId,
    pub expected_revision: DocumentRevision,
}
