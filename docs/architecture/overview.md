# Architecture Overview

## Planes

| Plane | Responsibility | Foundation status |
| --- | --- | --- |
| Performance | Audio, jitter, recording | Docs/contracts only |
| Composition | Document ops, validation, replay | Implemented in Rust core |
| Session | Identity, epoch, capabilities | Minimal types |

## Crates

- `echlub-model` — canonical types
- `echlub-kernel` — validate/apply/hash
- `echlub-collaboration` — operation relation classification
- `echlub-protocol` — transport-independent frames/flows
- `echlub-replication` — in-memory replica lab
- `echlub-session` — capability contracts
- `echlub-web` — WASM facade

## Apps

- `apps/web` — React lab UI (presentation only)
- `apps/control-plane` — health/capabilities skeleton
- `apps/protocol-lab` — deterministic scenario runner

WebRTC is a future baseline adapter candidate, not current domain truth.
