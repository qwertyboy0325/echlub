# EchLub

Experimental music-systems foundation and performance evidence lab.

EchLub combines a Rust semantic-composition core, a React/WASM laboratory UI,
and a bounded WebSocket/WebRTC measurement environment. It explores explicit
musical state, transport-independent protocol boundaries, and browser
performance evidence before claiming a product direction.

## What exists now

- Deterministic semantic operations, validation, and state hashing in Rust.
- Transport-independent protocol, session, and in-memory replication contracts.
- A React + Vite / WASM laboratory UI and a small control plane with WebSocket
  signaling.
- Exploratory WebRTC instrumentation, validated evidence schemas, and a
  reproducible browser-performance harness.

## Scope and evidence boundary

This is a technical foundation, not a production DAW or an accepted product
thesis. It does not claim latency superiority, acoustic mouth-to-ear
measurement, a complete CRDT or musical conflict-resolution model, or a final
transport choice.

See the [architecture overview](docs/architecture/overview.md),
[performance baseline](evidence/performance-baseline/README.md), and
[live two-peer evidence boundary](evidence/live-two-peer/README.md) for the
implemented scope and evidence boundaries.

## Research history

EchLub does not currently claim collaborative composition as a selling point or
product differentiator. The earlier collaboration-led demo direction failed to
make collaboration perceptible to a cold viewer: cursor and presence activity
read as interface choreography, while scripted state transitions did not
establish musicians changing one another's work.

That history is retained as research evidence, not promoted as a successful
product demo. No replacement product thesis is selected here; low-latency
performance, Live recomposition, AI, and the Rust audio core are not implicitly
chosen next directions.

## Structure

- `crates/` — Rust domain core (model, kernel, protocol, replication, session,
  performance, web WASM)
- `apps/web/` — React + Vite lab UI (foundation + performance sections)
- `apps/control-plane/` — HTTP control plane with WebSocket signaling
- `apps/performance-report/` — evidence validate/summarize CLI
- `tools/performance-browser-harness/` — Puppeteer-based synthetic harness
- `evidence/performance-baseline/` — exploratory performance evidence and
  harness artifacts

## Quick start

```bash
corepack pnpm install
corepack pnpm verify
corepack pnpm performance:synthetic
```

## Reference repos

Use `corepack pnpm references:check` to verify SHAs against
`docs/reference/source-lock.json`.
