use std::fs;
use std::path::PathBuf;

use echlub_collaboration::classify;
use echlub_model::{
    envelope::PROTOCOL_VERSION, ActorId, AddNote, AddTrack, DeleteNote, DocumentId,
    DocumentRevision, DocumentState, MoveNote, NoteId, Operation, OperationEnvelope, OperationId,
    OperationRelation, Tick, TrackId, ValidationDecision,
};
use echlub_replication::Replica;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FoundationVector {
    schema_version: u32,
    hash_version: u32,
    operations: Vec<VectorOperation>,
    expected_decisions: Vec<String>,
    expected_final_hash: String,
}

#[derive(Debug, Deserialize)]
struct VectorOperation {
    operation_id: u64,
    actor_id: u64,
    actor_sequence: u64,
    operation: VectorOp,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum VectorOp {
    AddTrack {
        track_id: u64,
        name: String,
    },
    AddNote {
        track_id: u64,
        note_id: u64,
        start: u64,
        duration: u64,
        pitch: u8,
        velocity: u8,
    },
    MoveNote {
        note_id: u64,
        expected_revision: u64,
        new_start: u64,
    },
    DeleteNote {
        note_id: u64,
        expected_revision: u64,
    },
}

fn main() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-vectors/foundation-v1.json");
    let vector: FoundationVector =
        serde_json::from_str(&fs::read_to_string(&root).expect("read vector"))
            .expect("parse vector");

    let mut failures = Vec::new();
    let mut replica_a = Replica::new(DocumentState::empty());
    let mut replica_b = Replica::new(DocumentState::empty());

    let envelopes: Vec<OperationEnvelope> = vector
        .operations
        .iter()
        .map(|op| OperationEnvelope {
            protocol_version: PROTOCOL_VERSION,
            document_id: DocumentId(1),
            operation_id: OperationId(op.operation_id),
            actor_id: ActorId(op.actor_id),
            actor_sequence: op.actor_sequence,
            operation: match &op.operation {
                VectorOp::AddTrack { track_id, name } => Operation::AddTrack(AddTrack {
                    track_id: TrackId(*track_id),
                    name: name.clone(),
                }),
                VectorOp::AddNote {
                    track_id,
                    note_id,
                    start,
                    duration,
                    pitch,
                    velocity,
                } => Operation::AddNote(AddNote {
                    track_id: TrackId(*track_id),
                    note_id: NoteId(*note_id),
                    start: Tick(*start),
                    duration: Tick(*duration),
                    pitch: *pitch,
                    velocity: *velocity,
                }),
                VectorOp::MoveNote {
                    note_id,
                    expected_revision,
                    new_start,
                } => Operation::MoveNote(MoveNote {
                    note_id: NoteId(*note_id),
                    expected_revision: DocumentRevision(*expected_revision),
                    new_start: Tick(*new_start),
                }),
                VectorOp::DeleteNote {
                    note_id,
                    expected_revision,
                } => Operation::DeleteNote(DeleteNote {
                    note_id: NoteId(*note_id),
                    expected_revision: DocumentRevision(*expected_revision),
                }),
            },
        })
        .collect();

    // Step 1-2: create replicas, A creates track
    let track_op = &envelopes[0];
    replica_a.ingest(track_op);
    replica_b.ingest(track_op);

    // Step 3-4: distinct notes, different delivery order
    let note_a = &envelopes[1];
    let note_b = &envelopes[2];
    replica_a.ingest(note_a);
    replica_a.ingest(note_b);
    replica_b.ingest(note_b);
    replica_b.ingest(note_a);

    // Step 5-6: duplicate
    let duplicate = &envelopes[3];
    let dup_receipt = replica_a.ingest(duplicate);
    if dup_receipt.decision != ValidationDecision::Duplicate {
        failures.push("duplicate not suppressed".into());
    }

    // Step 7: convergence
    let converged = replica_a.checkpoint().state_hash == replica_b.checkpoint().state_hash;
    if !converged {
        failures.push("hashes did not converge".into());
    }

    // Step 8-9: concurrent moves same note
    let move_a = &envelopes[4];
    let move_b = &envelopes[5];
    if classify(move_a, move_b) != OperationRelation::RequiresCoordination {
        failures.push("same-note moves not classified RequiresCoordination".into());
    }
    let first_move = replica_a.ingest(move_a);
    let second_move = replica_a.ingest(move_b);
    if first_move.decision != ValidationDecision::Accepted {
        failures.push("expected first concurrent move to be accepted".into());
    }
    if second_move.decision == ValidationDecision::Accepted {
        failures.push("silent winner selected for concurrent moves".into());
    }

    // Step 10-11: stale revision
    let stale = &envelopes[6];
    let stale_receipt = replica_a.ingest(stale);
    if stale_receipt.decision != ValidationDecision::StaleRevision {
        failures.push("stale revision not rejected".into());
    }

    let actual_hash = hex_hash(replica_a.checkpoint().state_hash);
    if actual_hash != vector.expected_final_hash {
        failures.push(format!(
            "hash mismatch expected {} got {}",
            vector.expected_final_hash, actual_hash
        ));
    }

    for (idx, expected) in vector.expected_decisions.iter().enumerate() {
        if idx >= envelopes.len() {
            break;
        }
        let _ = expected;
    }

    let result = serde_json::json!({
        "schema_version": vector.schema_version,
        "hash_version": vector.hash_version,
        "converged": converged,
        "final_hash": actual_hash,
        "failures": failures,
        "passed": failures.is_empty(),
    });

    println!("{}", serde_json::to_string_pretty(&result).unwrap());

    if !failures.is_empty() {
        std::process::exit(1);
    }
}

fn hex_hash(hash: echlub_model::StateHash) -> String {
    hash.0.iter().map(|b| format!("{b:02x}")).collect()
}
