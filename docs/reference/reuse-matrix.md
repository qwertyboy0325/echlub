# Reuse Matrix

| Legacy artifact | Action | New target |
| --- | --- | --- |
| Music-arrangement Track/MidiNote types | Translate | `echlub-model` |
| Event-sourced TS repositories | Discard | replaced by kernel + replication lab |
| Tone.js / audio engines | Discard (foundation) | future performance plane |
| Collaboration WebSocket gateway | Discard (foundation) | protocol flows only |
| JWT auth module | Discard (foundation) | capability flags false |
| DAW React layout | Reference UI only | future UX packages |
| VoxProof Cursor rules/skills | Adapt | `.cursor/rules`, `.cursor/skills` |
| VoxProof persistence evidence workflow | Discard | too heavy for foundation |

## Explicit discards

- Copy-paste of legacy modules into `crates/` or `apps/web`
- `.reference/**` as build dependency
- ORM migrations and Docker compose from backend
