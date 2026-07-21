use echlub_kernel::Kernel;
use echlub_model::intent::{EditAction, EditIntent, EditTarget, IntentExpiry};
use echlub_model::{
    envelope::PROTOCOL_VERSION, ActorId, AddNote, AddTrack, DeleteNote, DocumentId,
    DocumentRevision, DocumentState, MoveNote, NoteId, Operation, OperationEnvelope, OperationId,
    Tick, TrackId, ValidationDecision,
};

fn envelope(op: Operation, op_id: u64, seq: u64) -> OperationEnvelope {
    OperationEnvelope {
        protocol_version: PROTOCOL_VERSION,
        document_id: DocumentId(1),
        operation_id: OperationId(op_id),
        actor_id: ActorId(1),
        actor_sequence: seq,
        operation: op,
    }
}

#[test]
fn add_track() {
    let mut state = DocumentState::empty();
    let env = envelope(
        Operation::AddTrack(AddTrack {
            track_id: TrackId(1),
            name: "Drums".into(),
        }),
        1,
        1,
    );
    assert_eq!(Kernel::validate(&state, &env), ValidationDecision::Accepted);
    Kernel::apply(&mut state, &env).unwrap();
    assert!(state.tracks.contains_key(&TrackId(1)));
}

#[test]
fn add_note() {
    let mut state = DocumentState::empty();
    Kernel::apply(
        &mut state,
        &envelope(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "Piano".into(),
            }),
            1,
            1,
        ),
    )
    .unwrap();
    let env = envelope(
        Operation::AddNote(AddNote {
            track_id: TrackId(1),
            note_id: NoteId(1),
            start: Tick(0),
            duration: Tick(480),
            pitch: 60,
            velocity: 100,
        }),
        2,
        2,
    );
    assert_eq!(Kernel::validate(&state, &env), ValidationDecision::Accepted);
    Kernel::apply(&mut state, &env).unwrap();
    assert_eq!(state.tracks[&TrackId(1)].notes.len(), 1);
}

#[test]
fn move_note_with_correct_revision() {
    let mut state = seeded_note_state();
    let note = state.tracks[&TrackId(1)].notes[&NoteId(1)].clone();
    let env = envelope(
        Operation::MoveNote(MoveNote {
            note_id: NoteId(1),
            expected_revision: note.revision,
            new_start: Tick(960),
        }),
        3,
        3,
    );
    assert_eq!(Kernel::validate(&state, &env), ValidationDecision::Accepted);
    Kernel::apply(&mut state, &env).unwrap();
    assert_eq!(state.tracks[&TrackId(1)].notes[&NoteId(1)].start, Tick(960));
}

#[test]
fn stale_revision_rejection() {
    let state = seeded_note_state();
    let env = envelope(
        Operation::MoveNote(MoveNote {
            note_id: NoteId(1),
            expected_revision: DocumentRevision(0),
            new_start: Tick(960),
        }),
        3,
        3,
    );
    assert_eq!(
        Kernel::validate(&state, &env),
        ValidationDecision::StaleRevision
    );
}

#[test]
fn delete_note() {
    let mut state = seeded_note_state();
    let rev = state.tracks[&TrackId(1)].notes[&NoteId(1)].revision;
    let env = envelope(
        Operation::DeleteNote(DeleteNote {
            note_id: NoteId(1),
            expected_revision: rev,
        }),
        4,
        4,
    );
    Kernel::apply(&mut state, &env).unwrap();
    assert!(!state.tracks[&TrackId(1)].notes.contains_key(&NoteId(1)));
}

#[test]
fn missing_track_rejection() {
    let state = DocumentState::empty();
    let env = envelope(
        Operation::AddNote(AddNote {
            track_id: TrackId(99),
            note_id: NoteId(1),
            start: Tick(0),
            duration: Tick(480),
            pitch: 60,
            velocity: 100,
        }),
        1,
        1,
    );
    assert_eq!(
        Kernel::validate(&state, &env),
        ValidationDecision::MissingDependency
    );
}

#[test]
fn missing_note_rejection() {
    let mut state = DocumentState::empty();
    Kernel::apply(
        &mut state,
        &envelope(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "Piano".into(),
            }),
            1,
            1,
        ),
    )
    .unwrap();
    let env = envelope(
        Operation::DeleteNote(DeleteNote {
            note_id: NoteId(99),
            expected_revision: DocumentRevision(1),
        }),
        2,
        2,
    );
    assert_eq!(
        Kernel::validate(&state, &env),
        ValidationDecision::MissingDependency
    );
}

#[test]
fn unsupported_protocol_version() {
    let state = DocumentState::empty();
    let mut env = envelope(
        Operation::AddTrack(AddTrack {
            track_id: TrackId(1),
            name: "Piano".into(),
        }),
        1,
        1,
    );
    env.protocol_version = 999;
    assert_eq!(
        Kernel::validate(&state, &env),
        ValidationDecision::UnsupportedVersion
    );
}

#[test]
fn deterministic_hash_for_equivalent_state() {
    let a = seeded_note_state();
    let b = seeded_note_state();
    assert_eq!(Kernel::normalized_hash(&a), Kernel::normalized_hash(&b));
}

#[test]
fn ephemeral_intent_excluded_from_hash() {
    let state = seeded_note_state();
    let hash_before = Kernel::normalized_hash(&state);
    let _intent = EditIntent {
        actor_id: ActorId(2),
        target: EditTarget::Note { note_id: NoteId(1) },
        action: EditAction::Move,
        expiry: IntentExpiry { sequence: 1 },
    };
    let hash_after = Kernel::normalized_hash(&state);
    assert_eq!(hash_before, hash_after);
}

fn seeded_note_state() -> DocumentState {
    let mut state = DocumentState::empty();
    Kernel::apply(
        &mut state,
        &envelope(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "Piano".into(),
            }),
            1,
            1,
        ),
    )
    .unwrap();
    Kernel::apply(
        &mut state,
        &envelope(
            Operation::AddNote(AddNote {
                track_id: TrackId(1),
                note_id: NoteId(1),
                start: Tick(0),
                duration: Tick(480),
                pitch: 60,
                velocity: 100,
            }),
            2,
            2,
        ),
    )
    .unwrap();
    state
}
