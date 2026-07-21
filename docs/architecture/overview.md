# Architecture Overview

## Planes

| Plane | Responsibility | Foundation status |
| --- | --- | --- |
| Performance | Audio, jitter, recording | Baseline lab (synthetic + live UI) |
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

## Apps

- `apps/web` — React lab UI (foundation + performance sections)
- `apps/control-plane` — health/capabilities + WebSocket signaling
- `apps/protocol-lab` — deterministic scenario runner
- `apps/performance-report` — evidence validate/summarize CLI

WebRTC is a baseline adapter for performance lab only, not transport selection truth.
