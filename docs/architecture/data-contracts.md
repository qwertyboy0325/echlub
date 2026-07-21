# Data Contracts

## Musical time

Integer ticks only in canonical state (`Tick(u64)`, `TimeBase`).

## Operations

`OperationEnvelope` carries `protocol_version`, IDs, actor metadata, and one of:

- `AddTrack`, `AddNote`, `MoveNote`, `DeleteNote`

## Validation

Explicit `ValidationDecision` variants; no silent overwrite on stale revision.

## Hashing

`CanonicalDocumentV1` + BLAKE3 over deterministic postcard bytes.

Excluded from hash: edit intent, UI/connection state, transport metadata.

## Intent

`EditIntent` is ephemeral and latest-value; not hashed.
