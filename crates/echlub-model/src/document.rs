use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::{NoteId, TrackId};
use crate::time::Tick;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeBase {
    pub ticks_per_quarter: u32,
}

impl Default for TimeBase {
    fn default() -> Self {
        Self {
            ticks_per_quarter: 480,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DocumentRevision(pub u64);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiNote {
    pub id: NoteId,
    pub start: Tick,
    pub duration: Tick,
    pub pitch: u8,
    pub velocity: u8,
    pub revision: DocumentRevision,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub notes: BTreeMap<NoteId, MidiNote>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentState {
    pub time_base: TimeBase,
    pub revision: DocumentRevision,
    pub tracks: BTreeMap<TrackId, Track>,
}

impl DocumentState {
    pub fn empty() -> Self {
        Self {
            time_base: TimeBase::default(),
            revision: DocumentRevision(0),
            tracks: BTreeMap::new(),
        }
    }
}
