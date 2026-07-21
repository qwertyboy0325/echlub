# Legacy Archaeology

Read-only inspection of shallow-cloned reference repositories (see `source-lock.json`).

## Source SHAs

| Repository | SHA |
| --- | --- |
| qwertyboy0325/echlub-front | `17d36ea7721bec7cb68460bb58c7e40ae9f9583c` |
| qwertyboy0325/echlub_backend | `b356fc2904bcf453c0109ffff3b70f480cafdccd` |
| qwertyboy0325/vox-proof | `17f62b6ea5b8908cc4ab32532ef7d72ce288cfbd` |

## Old EchLub vocabulary

- **Frontend (`echlub-front`)**: React DAW UI, music-arrangement module with Track/Clip/MidiNote aggregates, event-sourced repositories, Tone.js adapters, collaboration pages (rooms/WebRTC-oriented).
- **Backend (`echlub_backend`)**: Node/TypeScript modules for auth (JWT) and collaboration rooms (WebSocket gateway, TypeORM persistence).

## Reusable user flows (conceptual)

- Create/join collaboration room
- Arrange tracks and MIDI notes
- Transport-oriented session UI

## Reusable tests/fixtures

- Frontend Jest tests around undo/redo and event store (patterns only)
- Backend collaboration unit/integration tests (room lifecycle)

Do not copy these suites wholesale; translate invariants into Rust/WASM tests.

## Useful UI traces

- DAW layout components (track headers, transport bar) as future UX reference
- Collaboration room pages as session entry patterns

## Obsolete or dangerous architecture

- Duplicated domain logic across TypeScript modules and UI adapters
- Event store + ORM persistence entangled with live collaboration
- WebRTC/audio assumptions embedded in arrangement domain
- Random/event-time IDs inside client domain paths

## Frontend/backend contract mismatches

- Frontend arrangement events vs backend room/signaling APIs are loosely coupled
- No shared canonical operation envelope or deterministic hash contract

## Mapping to new crates/apps

| Legacy concept | New location |
| --- | --- |
| Track/MidiNote | `echlub-model`, `echlub-kernel` |
| Collaboration room | future session fabric + protocol |
| Web UI | `apps/web` via `echlub-web` WASM |
| Signaling gateway | future adapter (not implemented) |
| Auth module | out of scope |

## VoxProof governance patterns

See `docs/ai/voxproof-cursor-adaptation.md`.
