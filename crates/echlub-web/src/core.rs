use std::collections::BTreeSet;

use echlub_kernel::Kernel;
use echlub_model::{
    envelope::PROTOCOL_VERSION, ActorId, AddNote, AddTrack, DeleteNote, DocumentId,
    DocumentRevision, DocumentState, MoveNote, NoteId, Operation, OperationEnvelope, OperationId,
    Tick, TrackId, ValidationDecision,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct EchlubCore {
    state: DocumentState,
    document_id: DocumentId,
    actor_id: ActorId,
    seen: BTreeSet<OperationId>,
    next_op: u64,
    last_decision: ValidationDecision,
}

#[wasm_bindgen]
impl EchlubCore {
    #[wasm_bindgen(constructor)]
    pub fn new(document_id: u64, actor_id: u64) -> Self {
        Self {
            state: DocumentState::empty(),
            document_id: DocumentId(document_id),
            actor_id: ActorId(actor_id),
            seen: BTreeSet::new(),
            next_op: 1,
            last_decision: ValidationDecision::Accepted,
        }
    }

    pub fn add_track(&mut self, track_id: u64, name: &str) -> String {
        self.apply(Operation::AddTrack(AddTrack {
            track_id: TrackId(track_id),
            name: name.to_string(),
        }))
    }

    pub fn add_note(
        &mut self,
        track_id: u64,
        note_id: u64,
        start_tick: u64,
        duration_ticks: u64,
        pitch: u8,
        velocity: u8,
    ) -> String {
        self.apply(Operation::AddNote(AddNote {
            track_id: TrackId(track_id),
            note_id: NoteId(note_id),
            start: Tick(start_tick),
            duration: Tick(duration_ticks),
            pitch,
            velocity,
        }))
    }

    pub fn move_note(
        &mut self,
        note_id: u64,
        expected_revision: u64,
        new_start_tick: u64,
    ) -> String {
        self.apply(Operation::MoveNote(MoveNote {
            note_id: NoteId(note_id),
            expected_revision: DocumentRevision(expected_revision),
            new_start: Tick(new_start_tick),
        }))
    }

    pub fn delete_note(&mut self, note_id: u64, expected_revision: u64) -> String {
        self.apply(Operation::DeleteNote(DeleteNote {
            note_id: NoteId(note_id),
            expected_revision: DocumentRevision(expected_revision),
        }))
    }

    pub fn snapshot_json(&self) -> String {
        serde_json::to_string(&self.state).unwrap_or_else(|_| "{}".into())
    }

    pub fn state_hash_hex(&self) -> String {
        let hash = Kernel::normalized_hash(&self.state);
        hash.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn last_decision_json(&self) -> String {
        serde_json::to_string(&self.last_decision).unwrap_or_else(|_| "\"Rejected\"".into())
    }

    fn apply(&mut self, operation: Operation) -> String {
        let op_id = OperationId(self.next_op);
        self.next_op += 1;
        let envelope = OperationEnvelope {
            protocol_version: PROTOCOL_VERSION,
            document_id: self.document_id,
            operation_id: op_id,
            actor_id: self.actor_id,
            actor_sequence: self.next_op - 1,
            operation,
        };

        if self.seen.contains(&op_id) {
            self.last_decision = ValidationDecision::Duplicate;
        } else {
            let decision = Kernel::validate(&self.state, &envelope);
            self.last_decision = decision;
            if decision == ValidationDecision::Accepted {
                let _ = Kernel::apply(&mut self.state, &envelope);
                self.seen.insert(op_id);
            }
        }

        self.last_decision_json()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facade_initialization() {
        let core = EchlubCore::new(1, 1);
        assert_eq!(core.state_hash_hex().len(), 64);
    }

    #[test]
    fn one_accepted_operation() {
        let mut core = EchlubCore::new(1, 1);
        let decision = core.add_track(1, "Piano");
        assert!(decision.contains("Accepted"));
    }

    #[test]
    fn one_rejected_operation() {
        let mut core = EchlubCore::new(1, 1);
        core.add_note(99, 1, 0, 480, 60, 100);
        assert!(core.last_decision_json().contains("MissingDependency"));
    }

    #[test]
    fn stable_snapshot_hash_output() {
        let mut a = EchlubCore::new(1, 1);
        let mut b = EchlubCore::new(1, 1);
        a.add_track(1, "Piano");
        b.add_track(1, "Piano");
        assert_eq!(a.state_hash_hex(), b.state_hash_hex());
        assert!(!a.snapshot_json().is_empty());
    }
}
