use echlub_model::{DocumentState, Operation, OperationEnvelope, ValidationDecision};

pub fn validate(state: &DocumentState, envelope: &OperationEnvelope) -> ValidationDecision {
    if envelope.protocol_version != echlub_model::envelope::PROTOCOL_VERSION {
        return ValidationDecision::UnsupportedVersion;
    }

    match &envelope.operation {
        Operation::AddTrack(op) => {
            if state.tracks.contains_key(&op.track_id) {
                ValidationDecision::Rejected
            } else {
                ValidationDecision::Accepted
            }
        }
        Operation::AddNote(op) => {
            if !state.tracks.contains_key(&op.track_id) {
                return ValidationDecision::MissingDependency;
            }
            if op.duration.0 == 0 || op.pitch > 127 || op.velocity > 127 {
                return ValidationDecision::Rejected;
            }
            if let Some(track) = state.tracks.get(&op.track_id) {
                if track.notes.contains_key(&op.note_id) {
                    return ValidationDecision::Rejected;
                }
            }
            ValidationDecision::Accepted
        }
        Operation::MoveNote(op) => match find_note(state, op.note_id) {
            None => ValidationDecision::MissingDependency,
            Some(note) if note.revision != op.expected_revision => {
                ValidationDecision::StaleRevision
            }
            _ => ValidationDecision::Accepted,
        },
        Operation::DeleteNote(op) => match find_note(state, op.note_id) {
            None => ValidationDecision::MissingDependency,
            Some(note) if note.revision != op.expected_revision => {
                ValidationDecision::StaleRevision
            }
            _ => ValidationDecision::Accepted,
        },
    }
}

fn find_note(
    state: &DocumentState,
    note_id: echlub_model::NoteId,
) -> Option<echlub_model::MidiNote> {
    state
        .tracks
        .values()
        .find_map(|track| track.notes.get(&note_id).cloned())
}

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("validation failed")]
    ValidationFailed,
}
