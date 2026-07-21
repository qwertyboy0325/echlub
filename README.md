# EchLub

Collaborative music production platform — greenfield rewrite.

## Structure

- `crates/` — Rust domain core (model, kernel, protocol, replication, session, web WASM)
- `apps/web/` — React + Vite lab UI
- `apps/control-plane/` — HTTP control plane
- `apps/protocol-lab/` — Deterministic protocol scenario runner
- `test-vectors/` — Canonical test fixtures
- `docs/` — Architecture, quality, and reference archaeology

## Quick start

```bash
corepack enable
pnpm install
pnpm verify
```

## Reference repos

Local read-only clones live under `.reference/` (gitignored). Use `pnpm references:check` to verify SHAs against `source-lock.json`.
