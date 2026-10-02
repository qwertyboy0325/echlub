# EchLub

Real-time jamming for musicians across Taiwan: a Rust jam core, a relay
prototype, simulated musicians, and a performance evidence lab.

## Current goal

Let musicians in different parts of Taiwan play together live. First target:
rock, 2–4 players, with a 4-piece as the final trial for this phase. Entry
should feel like one simple step (browser preferred, not mandated). See
[vision](docs/product/vision.md) and
[ADR-0006](docs/decisions/ADR-0006-taiwan-realtime-jam.md).

## What exists now

- `echlub-jam`: platform-neutral jam core (packets, jitter buffer,
  mix-minus, impairment model, latency budget, pipeline simulator).
- `echlub-musician-sim`: simulated rock musicians that keep time from what
  they hear through the network, plus a synth to listen to the result.
- `jam-relay` (UDP, `mix` / `forward`) and `jam-lab` (scenarios, sweeps,
  real-UDP bot band).
- Earlier foundations: deterministic semantic composition core, WebRTC
  performance lab with validated evidence schemas, control plane.

## Evidence boundary

Jam results so far are **simulations with assumed inputs**, not measurements.
No latency claim, final transport, or topology choice is made. See
[realtime jam architecture](docs/architecture/realtime-jam.md) and
[simulation outputs](evidence/realtime-jam-simulation/README.md).

## Research history

An earlier collaboration-led *composition* demo failed to make collaboration
perceptible to a cold viewer; it is retained as research evidence. Real-time
playing was chosen next partly because whether a band stays together is
audible to anyone.

## Structure

- `crates/` — Rust core (jam, musician-sim, model, kernel, protocol,
  replication, session, performance, web WASM)
- `apps/web/` — React + Vite lab UI (foundation + performance sections)
- `apps/control-plane/` — HTTP control plane with WebSocket signaling
- `apps/performance-report/` — evidence validate/summarize CLI
- `apps/jam-relay/` — UDP jam relay prototype
- `apps/jam-lab/` — jam scenarios, sweeps, bot musicians
- `tools/performance-browser-harness/` — Puppeteer-based synthetic harness
- `evidence/performance-baseline/` — exploratory performance evidence and
  harness artifacts

## Quick start

```bash
corepack pnpm install
corepack pnpm verify
corepack pnpm performance:synthetic

# real-time jam lab
cargo run -p echlub-jam-lab --bin jam-lab -- scenario --preset taiwan-rock-4-wired --topology forward
cargo run -p echlub-jam-lab --bin jam-lab -- sweep
cargo run -p echlub-jam-lab --bin jam-lab -- bots --count 4 --mode probe --impair
cargo run -p echlub-jam-relay --bin jam-relay -- --bind 0.0.0.0:7400 --mode forward
```

## Reference repos

Use `corepack pnpm references:check` to verify SHAs against
`docs/reference/source-lock.json`.
