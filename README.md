# EchLub

Collaborative music production platform — **foundation + performance baseline** scaffold for a greenfield rewrite.

No mouth-to-ear, production-readiness, CRDT-completeness, or transport-selection claims are demonstrated.

Legacy repositories are external read-only references under `.reference/` (gitignored).

## Structure

- `crates/` — Rust domain core (model, kernel, protocol, replication, session, performance, web WASM)
- `apps/web/` — React + Vite lab UI (foundation + performance sections)
- `apps/control-plane/` — HTTP control plane with WebSocket signaling
- `apps/performance-report/` — evidence validate/summarize CLI
- `tools/performance-browser-harness/` — puppeteer-core synthetic harness
- `evidence/performance-baseline/` — validated performance observations

## Quick start

```bash
corepack pnpm install
corepack pnpm verify
corepack pnpm performance:synthetic
```

## Reference repos

Use `corepack pnpm references:check` to verify SHAs against `docs/reference/source-lock.json`.
