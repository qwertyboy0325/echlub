# ADR-0005 Intent-Aware Semantic Replication

## Decision

Use explicit operation relations and coordination escalation instead of silent CRDT merges.

## Consequences

Concurrent same-note edits require coordination; delete/move conflicts are destructive conflicts.

No general-purpose CRDT dependency in foundation.
