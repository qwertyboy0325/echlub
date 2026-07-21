# ECHLUB-REPOSITORY-CONSOLIDATION-REMOTE-SYNC-01

## Objective

Consolidate local main with Cursor multitask setup branch, record owner-accepted probe verdict, validate baseline, and push main to origin with CI verification.

## Risk tier

Low — documentation and git sync only; no product code changes.

## Authorized file scope

- `docs/quality/multitask-smoke-probe.md`
- `docs/ai/cursor-multitask-and-model-routing.md`
- `docs/ai/model-budget.md`
- `docs/quality/bootstrap-known-issues.md`
- `docs/work-packages/ECHLUB-REPOSITORY-CONSOLIDATION-REMOTE-SYNC-01.md`
- `AGENTS.md` (only if needed)

## Forbidden scope

- `apps/**`, `crates/**`, `evidence/**`, `manifests/**`
- Product code, evidence manifests, force push, rebase, amend
- Non-main push, Grok re-invocation

## Preflight

- Git topology: ff-only merge `chore/cursor-multitask-routing` into `main`
- `python3 scripts/check-references.py`

## Required checks

1. `git diff --check`
2. `python3 scripts/check-references.py`
3. `python3 scripts/verify.py`
4. `corepack pnpm performance:test`
5. `python3 scripts/run-performance-baseline.py --validate-only`
6. Remote gate: ff-only push (`merge-base == origin/main`, `main..origin` empty)
7. `git push origin main:main`
8. CI watch on pushed commit

## 10 — Probe result (owner-accepted)

```yaml
probe_status: FAIL
failure_classification: MODEL_IDENTITY_UNVERIFIABLE
multitask_parallel_execution: verified
custom_agent_discovery: verified
grok_requested_model: cursor-grok-4.5-high-fast
grok_actual_model: unknown
grok_model_identity_verifiable: false
grok_routing_status: unverified
cursor_version: not recorded
cursor_surface: not recorded
parallelism_overlapping_pairs: not recorded
maximum_overlap_seconds: not recorded
request_ids: not recorded
api_quota_used: false
grok_invoked: false
```

Do not record Grok routing PASS. Multitask and custom-agent discovery are verified.

## Stop gate

`REMOTE_MAIN_SYNCED` on PASS; PARTIAL if push succeeds but CI pending/failed; BLOCKED if validation or push fails.

## 20 — Coordinator completion packet

Filled by coordinator phase after execution. See final report YAML emitted at package completion.
