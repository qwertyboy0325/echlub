# EchLub

Collaborative music production platform — **foundation scaffold** for a greenfield rewrite.

No low-latency, production-readiness, CRDT-completeness, or transport-selection claims are demonstrated in this package.

Legacy repositories are external read-only references under `.reference/` (gitignored).

## Structure

- `crates/` — Rust domain core (model, kernel, protocol, replication, session, web WASM)
- `apps/web/` — React + Vite lab UI
- `apps/control-plane/` — HTTP control plane skeleton
- `apps/protocol-lab/` — Deterministic protocol scenario runner
- `test-vectors/` — Canonical test fixtures
- `docs/` — Architecture, quality, and reference archaeology

## Quick start

```bash
corepack pnpm install
corepack pnpm verify
```

## Reference repos

Use `corepack pnpm references:check` to verify SHAs against `docs/reference/source-lock.json`.
