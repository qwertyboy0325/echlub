use echlub_kernel::Kernel;
use echlub_model::{
    envelope::PROTOCOL_VERSION, AddNote, AddTrack, DocumentId, DocumentState, NoteId, Operation,
    OperationEnvelope, OperationId, Tick, TrackId, ValidationDecision,
};

fn envelope(op: Operation) -> OperationEnvelope {
    OperationEnvelope {
        protocol_version: PROTOCOL_VERSION,
        document_id: DocumentId(1),
        operation_id: OperationId(1),
        actor_id: echlub_model::ActorId(1),
        actor_sequence: 1,
        operation: op,
    }
}

#[test]
fn rejects_zero_duration() {
    let mut state = DocumentState::empty();
    Kernel::apply(
        &mut state,
        &envelope(Operation::AddTrack(AddTrack {
            track_id: TrackId(1),
            name: "Piano".into(),
        })),
    )
    .unwrap();
    let env = envelope(Operation::AddNote(AddNote {
        track_id: TrackId(1),
        note_id: NoteId(1),
        start: Tick(0),
        duration: Tick(0),
        pitch: 60,
        velocity: 100,
    }));
    assert_eq!(Kernel::validate(&state, &env), ValidationDecision::Rejected);
}

#[test]
fn rejects_invalid_pitch() {
    let mut state = DocumentState::empty();
    Kernel::apply(
        &mut state,
        &envelope(Operation::AddTrack(AddTrack {
            track_id: TrackId(1),
            name: "Piano".into(),
        })),
    )
    .unwrap();
    let env = envelope(Operation::AddNote(AddNote {
        track_id: TrackId(1),
        note_id: NoteId(1),
        start: Tick(0),
        duration: Tick(480),
        pitch: 128,
        velocity: 100,
    }));
    assert_eq!(Kernel::validate(&state, &env), ValidationDecision::Rejected);
}

#[test]
fn rejects_invalid_velocity() {
    let mut state = DocumentState::empty();
    Kernel::apply(
        &mut state,
        &envelope(Operation::AddTrack(AddTrack {
            track_id: TrackId(1),
            name: "Piano".into(),
        })),
    )
    .unwrap();
    let env = envelope(Operation::AddNote(AddNote {
        track_id: TrackId(1),
        note_id: NoteId(1),
        start: Tick(0),
        duration: Tick(480),
        pitch: 60,
        velocity: 128,
    }));
    assert_eq!(Kernel::validate(&state, &env), ValidationDecision::Rejected);
}

#[test]
fn rejects_duplicate_track_ids() {
    let mut state = DocumentState::empty();
    Kernel::apply(
        &mut state,
        &envelope(Operation::AddTrack(AddTrack {
            track_id: TrackId(1),
            name: "Piano".into(),
        })),
    )
    .unwrap();
    assert_eq!(
        Kernel::validate(
            &state,
            &envelope(Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "Piano".into(),
            }))
        ),
        ValidationDecision::Rejected
    );
}
