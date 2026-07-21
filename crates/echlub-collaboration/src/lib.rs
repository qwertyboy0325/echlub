use echlub_model::{Operation, OperationEnvelope, OperationRelation};

pub fn classify(a: &OperationEnvelope, b: &OperationEnvelope) -> OperationRelation {
    use Operation::*;
    match (&a.operation, &b.operation) {
        (AddTrack(x), AddTrack(y)) if x.track_id != y.track_id => OperationRelation::Commutative,
        (AddTrack(_), AddTrack(_)) => OperationRelation::RequiresCoordination,
        (AddNote(x), AddNote(y)) if x.note_id != y.note_id => OperationRelation::Commutative,
        (AddNote(x), AddNote(y)) if x.note_id == y.note_id => {
            OperationRelation::RequiresCoordination
        }
        (MoveNote(x), MoveNote(y)) if x.note_id != y.note_id => OperationRelation::Independent,
        (MoveNote(x), MoveNote(y)) if x.note_id == y.note_id => {
            OperationRelation::RequiresCoordination
        }
        (DeleteNote(x), MoveNote(y)) | (MoveNote(y), DeleteNote(x)) if x.note_id == y.note_id => {
            OperationRelation::DestructiveConflict
        }
        (DeleteNote(x), DeleteNote(y)) if x.note_id == y.note_id => {
            OperationRelation::RequiresCoordination
        }
        _ => OperationRelation::Independent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use echlub_model::{
        envelope::PROTOCOL_VERSION, ActorId, AddTrack, DeleteNote, DocumentId, MoveNote, NoteId,
        OperationId, Tick, TrackId,
    };

    fn env(op: Operation, id: u64) -> OperationEnvelope {
        OperationEnvelope {
            protocol_version: PROTOCOL_VERSION,
            document_id: DocumentId(1),
            operation_id: OperationId(id),
            actor_id: ActorId(id),
            actor_sequence: id,
            operation: op,
        }
    }

    #[test]
    fn distinct_additions_commute() {
        let a = env(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "A".into(),
            }),
            1,
        );
        let b = env(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(2),
                name: "B".into(),
            }),
            2,
        );
        assert_eq!(classify(&a, &b), OperationRelation::Commutative);
    }

    #[test]
    fn distinct_note_moves_are_independent() {
        let a = env(
            Operation::MoveNote(MoveNote {
                note_id: NoteId(1),
                expected_revision: echlub_model::DocumentRevision(1),
                new_start: Tick(10),
            }),
            1,
        );
        let b = env(
            Operation::MoveNote(MoveNote {
                note_id: NoteId(2),
                expected_revision: echlub_model::DocumentRevision(1),
                new_start: Tick(20),
            }),
            2,
        );
        assert_eq!(classify(&a, &b), OperationRelation::Independent);
    }

    #[test]
    fn same_note_concurrent_moves_require_coordination() {
        let a = env(
            Operation::MoveNote(MoveNote {
                note_id: NoteId(1),
                expected_revision: echlub_model::DocumentRevision(1),
                new_start: Tick(10),
            }),
            1,
        );
        let b = env(
            Operation::MoveNote(MoveNote {
                note_id: NoteId(1),
                expected_revision: echlub_model::DocumentRevision(1),
                new_start: Tick(20),
            }),
            2,
        );
        assert_eq!(classify(&a, &b), OperationRelation::RequiresCoordination);
    }

    #[test]
    fn delete_move_same_note_is_destructive_conflict() {
        let delete = env(
            Operation::DeleteNote(DeleteNote {
                note_id: NoteId(1),
                expected_revision: echlub_model::DocumentRevision(1),
            }),
            1,
        );
        let mv = env(
            Operation::MoveNote(MoveNote {
                note_id: NoteId(1),
                expected_revision: echlub_model::DocumentRevision(1),
                new_start: Tick(20),
            }),
            2,
        );
        assert_eq!(
            classify(&delete, &mv),
            OperationRelation::DestructiveConflict
        );
    }
}
