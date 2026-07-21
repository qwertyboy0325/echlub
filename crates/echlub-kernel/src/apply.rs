use echlub_model::{
    DocumentRevision, DocumentState, MidiNote, Operation, OperationEnvelope, Track,
    ValidationDecision,
};

use crate::validate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyReceipt {
    pub decision: ValidationDecision,
    pub new_revision: DocumentRevision,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ApplyError {
    #[error("duplicate operation")]
    Duplicate,
    #[error("validation decision: {0:?}")]
    Rejected(ValidationDecision),
}

pub fn apply(
    state: &mut DocumentState,
    envelope: &OperationEnvelope,
) -> Result<ApplyReceipt, ApplyError> {
    let decision = validate::validate(state, envelope);
    match decision {
        ValidationDecision::Accepted => {}
        ValidationDecision::Duplicate => return Err(ApplyError::Duplicate),
        other => return Err(ApplyError::Rejected(other)),
    }

    match &envelope.operation {
        Operation::AddTrack(op) => {
            state.tracks.insert(
                op.track_id,
                Track {
                    id: op.track_id,
                    name: op.name.clone(),
                    notes: Default::default(),
                },
            );
        }
        Operation::AddNote(op) => {
            let track = state.tracks.get_mut(&op.track_id).expect("validated");
            track.notes.insert(
                op.note_id,
                MidiNote {
                    id: op.note_id,
                    start: op.start,
                    duration: op.duration,
                    pitch: op.pitch,
                    velocity: op.velocity,
                    revision: DocumentRevision(1),
                },
            );
        }
        Operation::MoveNote(op) => {
            for track in state.tracks.values_mut() {
                if let Some(note) = track.notes.get_mut(&op.note_id) {
                    note.start = op.new_start;
                    note.revision = DocumentRevision(note.revision.0 + 1);
                    break;
                }
            }
        }
        Operation::DeleteNote(op) => {
            for track in state.tracks.values_mut() {
                if track.notes.remove(&op.note_id).is_some() {
                    break;
                }
            }
        }
    }

    sync_document_revision(state);
    Ok(ApplyReceipt {
        decision: ValidationDecision::Accepted,
        new_revision: state.revision,
    })
}

fn sync_document_revision(state: &mut DocumentState) {
    let max_rev = state
        .tracks
        .values()
        .flat_map(|track| track.notes.values())
        .map(|note| note.revision.0)
        .max()
        .unwrap_or(0);
    state.revision = DocumentRevision(max_rev);
}
