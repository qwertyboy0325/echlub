use std::collections::{BTreeMap, BTreeSet};

use echlub_kernel::Kernel;
use echlub_model::{ActorId, DocumentState, OperationEnvelope, OperationId, ValidationDecision};
use echlub_protocol::{CheckpointFrame, OperationBatch, ValidationReceipt};

#[derive(Debug, Clone)]
pub struct Replica {
    pub state: DocumentState,
    seen_operations: BTreeSet<OperationId>,
    actor_sequences: BTreeMap<ActorId, u64>,
    last_decision: ValidationDecision,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SequenceGap {
    pub actor_id: ActorId,
    pub expected: u64,
    pub received: u64,
}

impl Replica {
    pub fn new(state: DocumentState) -> Self {
        Self {
            state,
            seen_operations: BTreeSet::new(),
            actor_sequences: BTreeMap::new(),
            last_decision: ValidationDecision::Accepted,
        }
    }

    pub fn ingest(&mut self, envelope: &OperationEnvelope) -> ValidationReceipt {
        if self.seen_operations.contains(&envelope.operation_id) {
            self.last_decision = ValidationDecision::Duplicate;
            return ValidationReceipt {
                operation_id: envelope.operation_id,
                decision: ValidationDecision::Duplicate,
            };
        }

        let decision = Kernel::validate(&self.state, envelope);
        self.last_decision = decision;
        if decision == ValidationDecision::Accepted {
            let _ = Kernel::apply(&mut self.state, envelope);
            self.seen_operations.insert(envelope.operation_id);
            self.track_sequence(envelope.actor_id, envelope.actor_sequence);
        }

        ValidationReceipt {
            operation_id: envelope.operation_id,
            decision,
        }
    }

    pub fn ingest_batch(&mut self, batch: &OperationBatch) -> Vec<ValidationReceipt> {
        batch.operations.iter().map(|op| self.ingest(op)).collect()
    }

    pub fn replay(operations: &[OperationEnvelope]) -> Self {
        let mut replica = Replica::new(DocumentState::empty());
        for op in operations {
            replica.ingest(op);
        }
        replica
    }

    pub fn checkpoint(&self) -> CheckpointFrame {
        CheckpointFrame {
            state_hash: Kernel::normalized_hash(&self.state),
            revision: self.state.revision.0,
        }
    }

    pub fn sequence_gaps(&self, envelope: &OperationEnvelope) -> Option<SequenceGap> {
        let last = self
            .actor_sequences
            .get(&envelope.actor_id)
            .copied()
            .unwrap_or(0);
        if envelope.actor_sequence > last + 1 {
            Some(SequenceGap {
                actor_id: envelope.actor_id,
                expected: last + 1,
                received: envelope.actor_sequence,
            })
        } else {
            None
        }
    }

    fn track_sequence(&mut self, actor_id: ActorId, sequence: u64) {
        let entry = self.actor_sequences.entry(actor_id).or_insert(0);
        if sequence > *entry {
            *entry = sequence;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use echlub_model::{
        envelope::PROTOCOL_VERSION, AddNote, AddTrack, DocumentId, NoteId, Operation, OperationId,
        Tick, TrackId,
    };

    fn env(op: Operation, id: u64, actor: u64, seq: u64) -> OperationEnvelope {
        OperationEnvelope {
            protocol_version: PROTOCOL_VERSION,
            document_id: DocumentId(1),
            operation_id: OperationId(id),
            actor_id: ActorId(actor),
            actor_sequence: seq,
            operation: op,
        }
    }

    #[test]
    fn duplicate_suppression() {
        let mut replica = Replica::new(DocumentState::empty());
        let op = env(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "A".into(),
            }),
            1,
            1,
            1,
        );
        assert_eq!(replica.ingest(&op).decision, ValidationDecision::Accepted);
        assert_eq!(replica.ingest(&op).decision, ValidationDecision::Duplicate);
        assert_eq!(replica.seen_operations.len(), 1);
    }

    #[test]
    fn actor_sequence_gap_classification() {
        let replica = Replica::new(DocumentState::empty());
        let gap = env(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "A".into(),
            }),
            1,
            1,
            3,
        );
        let gap_info = replica.sequence_gaps(&gap).unwrap();
        assert_eq!(gap_info.expected, 1);
        assert_eq!(gap_info.received, 3);
    }

    #[test]
    fn replay_produces_same_state() {
        let ops = vec![
            env(
                Operation::AddTrack(AddTrack {
                    track_id: TrackId(1),
                    name: "A".into(),
                }),
                1,
                1,
                1,
            ),
            env(
                Operation::AddNote(AddNote {
                    track_id: TrackId(1),
                    note_id: NoteId(1),
                    start: Tick(0),
                    duration: Tick(480),
                    pitch: 60,
                    velocity: 100,
                }),
                2,
                1,
                2,
            ),
        ];
        let mut live = Replica::new(DocumentState::empty());
        for op in &ops {
            live.ingest(op);
        }
        let replayed = Replica::replay(&ops);
        assert_eq!(
            Kernel::normalized_hash(&live.state),
            Kernel::normalized_hash(&replayed.state)
        );
    }

    #[test]
    fn commutative_operations_converge() {
        let a1 = env(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "A".into(),
            }),
            1,
            1,
            1,
        );
        let a2 = env(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(2),
                name: "B".into(),
            }),
            2,
            2,
            1,
        );
        let mut r1 = Replica::new(DocumentState::empty());
        r1.ingest(&a1);
        r1.ingest(&a2);
        let mut r2 = Replica::new(DocumentState::empty());
        r2.ingest(&a2);
        r2.ingest(&a1);
        assert_eq!(r1.checkpoint().state_hash, r2.checkpoint().state_hash);
    }

    #[test]
    fn checkpoint_hashes_match() {
        let mut replica = Replica::new(DocumentState::empty());
        replica.ingest(&env(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "A".into(),
            }),
            1,
            1,
            1,
        ));
        let cp = replica.checkpoint();
        assert_eq!(cp.state_hash, Kernel::normalized_hash(&replica.state));
    }

    #[test]
    fn conflicting_same_note_not_silently_accepted() {
        let mut replica = Replica::new(DocumentState::empty());
        replica.ingest(&env(
            Operation::AddTrack(AddTrack {
                track_id: TrackId(1),
                name: "A".into(),
            }),
            1,
            1,
            1,
        ));
        replica.ingest(&env(
            Operation::AddNote(AddNote {
                track_id: TrackId(1),
                note_id: NoteId(1),
                start: Tick(0),
                duration: Tick(480),
                pitch: 60,
                velocity: 100,
            }),
            2,
            1,
            2,
        ));
        let rev = replica.state.tracks[&TrackId(1)].notes[&NoteId(1)].revision;
        let move_a = env(
            Operation::MoveNote(echlub_model::MoveNote {
                note_id: NoteId(1),
                expected_revision: rev,
                new_start: Tick(10),
            }),
            3,
            1,
            3,
        );
        let move_b = env(
            Operation::MoveNote(echlub_model::MoveNote {
                note_id: NoteId(1),
                expected_revision: rev,
                new_start: Tick(20),
            }),
            4,
            2,
            1,
        );
        let first = replica.ingest(&move_a);
        assert_eq!(first.decision, ValidationDecision::Accepted);
        let second = replica.ingest(&move_b);
        assert_eq!(second.decision, ValidationDecision::StaleRevision);
    }
}
