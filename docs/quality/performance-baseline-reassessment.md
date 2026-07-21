# Performance Baseline Reassessment

Reassessment: ECHLUB-PERFORMANCE-LAB-REAUDIT-CORRECTION-01

## Original synthetic artifacts

| Artifact | Classification |
| --- | --- |
| `evidence/performance-baseline/synthetic-2026-07-21T08-29-29-791Z.json` | FAILED_OBSERVATION |
| `evidence/performance-baseline/synthetic-2026-07-21T08-29-43-818Z.json` | FAILED_OBSERVATION |

## Rationale

Both artifacts are structurally valid JSON passing schema validation but semantically empty:

- `pulsesDetected: 0`, `detectionRate: 0`
- `bytesReceived: 0` while `bytesSent > 0` (inbound media path dead)
- Pulse timing metrics zero-filled as `observed: 0` instead of `unavailable`
- Derived report produced invalid negative `end_to_end_synthetic_ms`

These artifacts are **retained** for historical traceability and reclassified as `FAILED_OBSERVATION`. They must not be cited as successful baseline evidence.

## Corrected evidence location

Valid post-correction synthetic observations belong under:

`evidence/performance-baseline/corrected/<run-id>/`

Each run directory must pass both `validate` and `assess-synthetic`.

## Status

Pending corrected synthetic rerun (Phase D).
