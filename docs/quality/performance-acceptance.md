# Performance Baseline Acceptance

## Required checks

1. `pnpm verify` — full foundation + performance crate tests
2. `pnpm performance:test` — performance unit + signaling tests
3. `pnpm performance:synthetic` — browser harness (PASS if browser available)
4. Evidence validation via `echlub-performance-report validate`

## Explicit non-claims

- No mouth-to-ear acoustic measurement
- No transport selection or superiority claim
- No production readiness claim
- `evidence_status: exploratory_non_authoritative` on all artifacts

## PASS criteria

- All verify steps pass
- System browser synthetic run produces validated evidence artifact
- Audit: no critical/high findings

## PARTIAL criteria

- All verify steps pass
- No system browser available for synthetic run
