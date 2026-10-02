# EchLub Agent Guide

Cross-agent entry for the EchLub rewrite monorepo.

## Product

Real-time jamming for musicians across Taiwan (2–4 players, rock first; ADR-0006). Entry should feel like one simple step; browser preferred, not mandated. `echlub-jam` (platform-neutral jam core), `echlub-musician-sim` (simulated musicians), `jam-relay`, and `jam-lab` are the active prototype. The composition core and WebRTC performance lab are earlier foundations; composition is parked.

## Proof of behavior

Tests and `scripts/verify.py` are authoritative over documentation summaries.

## Required reading order

1. `README.md`
2. `docs/README.md`
3. `docs/product/vision.md`
4. `docs/architecture/overview.md`
5. Active work package under `docs/work-packages/` (currently `ECHLUB-REALTIME-JAM-PROTOTYPE-01.md`)
6. Applicable `.cursor/rules/`
7. Applicable `.cursor/skills/`
8. `docs/reference/legacy-archaeology.md` only when legacy context is needed
9. `docs/ai/cursor-multitask-and-model-routing.md` when using Multitask or custom subagents

## Cursor Multitask and custom agents

- Actual Multitask requires `/multitask` from the Agents Window or Plan → Build in Parallel; prompt wording alone is not activation proof.
- After changing `.cursor/agents/**`, reload the Cursor window and open a fresh chat before invoking new custom agents.
- Agent frontmatter `model:` is the requested model only; record actual runtime model identity for routing verification.
- See `docs/ai/cursor-multitask-and-model-routing.md` and `.cursor/rules/echlub-multitask-governance.mdc`.

## Reference repositories

`.reference/**` is read-only, gitignored, and must never become a build dependency.

## Scope boundaries

- No latency claim without evidence; simulation output (assumed profiles) is not evidence
- Keep `echlub-jam` and `echlub-musician-sim` platform-neutral and building for `wasm32-unknown-unknown`
- Simulated musicians are rule-based; no API-backed models
- No general CRDT claim
- No final transport selection
- API-backed models forbidden unless owner explicitly authorizes

## Model budget

See `docs/ai/model-budget.md`. Review date: 2026-08-11.

## Git authorization

Local commits only within authorized work packages. No remote, push, or history rewrite unless owner requests.
