# Performance Baseline Acceptance

> **Supersession note (2026-07-21):** Acceptance criteria expanded by ECHLUB-PERFORMANCE-LAB-REAUDIT-CORRECTION-01. Synthetic PASS now requires `assess-synthetic` in addition to `validate`. See `performance-lab-reaudit-report.md`.

## Required checks

1. `pnpm verify` — full foundation + performance crate tests
2. `pnpm performance:test` — performance unit + signaling tests
3. `pnpm performance:synthetic` — browser harness writing to `evidence/performance-baseline/corrected/<run-id>/`
4. Evidence validation via `echlub-performance-report validate`
5. Observation quality via `echlub-performance-report assess-synthetic`

## Explicit non-claims

- No mouth-to-ear acoustic measurement
- No transport selection or superiority claim
- No production readiness claim
- `evidence_status: exploratory_non_authoritative` on all artifacts

## PASS criteria

- All verify steps pass
- System browser synthetic run produces validated + assessed evidence under `corrected/<run-id>/`
- Audit: no critical/high findings

## PARTIAL criteria

- All verify steps pass
- No system browser available for synthetic run
