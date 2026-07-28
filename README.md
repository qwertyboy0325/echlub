# EchLub

Experimental music-systems foundation and performance evidence lab.

The current repository demonstrates bounded technical foundations:

- deterministic semantic operations over explicit musical document state;
- transport-independent protocol and session contracts;
- a Web UI and control-plane laboratory;
- exploratory WebRTC instrumentation and reproducible performance evidence.

## Product status

EchLub does not currently claim collaborative composition as a selling point or product differentiator.

The earlier collaboration-led demo direction failed to make collaboration perceptible to a cold viewer: cursor and presence activity read as interface choreography, while scripted state transitions did not establish musicians changing one another's work. That history is retained as research evidence, not promoted as a successful product demo.

No replacement product thesis is selected here. In particular, this repository does not claim:

- a production-ready DAW;
- a complete CRDT or musical conflict-resolution model;
- latency superiority or a final transport choice;
- that low-latency performance, Live recomposition, AI, or the Rust audio core is automatically the next product direction.

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
