# Model Budget

| Task type | Model |
| --- | --- |
| Implementation / scaffolding | Composer 2.5 Fast |
| One-shot archaeology | cursor-grok-4.5-high-fast |
| Final foundation audit | cursor-grok-4.5-high-fast |

## Runtime model identity policy

- Agent frontmatter `model:` is the **requested** model only.
- A Grok routing PASS requires runtime or execution metadata with `actual_model = cursor-grok-4.5-high-fast` and `model_identity_verifiable: true`.
- Agent prose claiming model identity is not proof.
- When runtime identity is unavailable, classify as `MODEL_IDENTITY_UNVERIFIABLE` — not PASS.
- Current probe status: Multitask verified; Grok routing unverified (`docs/quality/multitask-smoke-probe.md`).

## Forbidden by default

- API-backed models
- GPT-5.6 Sol High (requires future owner authorization)
- Repeated model sampling on the same writable task
- Expensive models rerunning deterministic terminal commands
- Claiming Grok routing PASS without verifiable runtime model metadata

## Bounded diagnostic exception

Cursor-native Grok may be invoked once for `ECHLUB-CURSOR-MULTITASK-PROBE-01` to verify custom-agent routing. This does not authorize API-backed models, repeated sampling, implementation, or product review.

## Review date

2026-08-11
