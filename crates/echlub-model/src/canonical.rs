use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::document::{MidiNote, TimeBase, Track};
use crate::ids::{NoteId, TrackId};

pub const CANONICAL_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalNoteV1 {
    pub id: NoteId,
    pub start: u64,
    pub duration: u64,
    pub pitch: u8,
    pub velocity: u8,
    pub revision: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalTrackV1 {
    pub id: TrackId,
    pub name: String,
    pub notes: BTreeMap<NoteId, CanonicalNoteV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalDocumentV1 {
    pub schema_version: u32,
    pub time_base_ticks_per_quarter: u32,
    pub revision: u64,
    pub tracks: BTreeMap<TrackId, CanonicalTrackV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateHash(pub [u8; 32]);

impl CanonicalDocumentV1 {
    pub fn from_state(state: &crate::document::DocumentState) -> Self {
        let tracks = state
            .tracks
            .iter()
            .map(|(track_id, track)| {
                let notes = track
                    .notes
                    .iter()
                    .map(|(note_id, note)| {
                        (
                            *note_id,
                            CanonicalNoteV1 {
                                id: note.id,
                                start: note.start.0,
                                duration: note.duration.0,
                                pitch: note.pitch,
                                velocity: note.velocity,
                                revision: note.revision.0,
                            },
                        )
                    })
                    .collect();
                (
                    *track_id,
                    CanonicalTrackV1 {
                        id: track.id,
                        name: track.name.clone(),
                        notes,
                    },
                )
            })
            .collect();

        Self {
            schema_version: CANONICAL_SCHEMA_VERSION,
            time_base_ticks_per_quarter: state.time_base.ticks_per_quarter,
            revision: state.revision.0,
            tracks,
        }
    }
}

impl From<&Track> for CanonicalTrackV1 {
    fn from(track: &Track) -> Self {
        let notes = track
            .notes
            .iter()
            .map(|(id, note)| (*id, CanonicalNoteV1::from(note)))
            .collect();
        Self {
            id: track.id,
            name: track.name.clone(),
            notes,
        }
    }
}

impl From<&MidiNote> for CanonicalNoteV1 {
    fn from(note: &MidiNote) -> Self {
        Self {
            id: note.id,
            start: note.start.0,
            duration: note.duration.0,
            pitch: note.pitch,
            velocity: note.velocity,
            revision: note.revision.0,
        }
    }
}

impl From<&TimeBase> for CanonicalDocumentV1 {
    fn from(_: &TimeBase) -> Self {
        unreachable!("use from_state instead")
    }
}
