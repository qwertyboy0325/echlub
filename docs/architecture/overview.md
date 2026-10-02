# Architecture Overview

## Planes

| Plane | Responsibility | Foundation status |
| --- | --- | --- |
| Performance | Audio, jitter, recording, real-time jam | Baseline lab (synthetic + live UI); jam core, relay prototype, simulators |
| Composition | Document ops, validation, replay | Implemented in Rust core |
| Session | Identity, epoch, capabilities | Minimal types |

## Crates

- `echlub-model` — canonical types
- `echlub-kernel` — validate/apply/hash
- `echlub-collaboration` — operation relation classification
- `echlub-protocol` — transport-independent frames/flows
- `echlub-replication` — in-memory replica lab
- `echlub-session` — capability contracts
- `echlub-performance` — evidence schema, validation, derived metrics
- `echlub-web` — WASM facade
- `echlub-jam` — platform-neutral real-time jam core (see `realtime-jam.md`)
- `echlub-musician-sim` — simulated rock musicians

## Apps

- `apps/web` — React lab UI (foundation + performance sections)
- `apps/control-plane` — health/capabilities + WebSocket signaling
- `apps/protocol-lab` — deterministic scenario runner
- `apps/performance-report` — evidence validate/summarize CLI
- `apps/jam-relay` — UDP jam relay prototype (`mix` / `forward`)
- `apps/jam-lab` — jam scenarios, latency sweeps, real-UDP bot musicians

WebRTC is a baseline adapter for performance lab only, not transport selection truth.
