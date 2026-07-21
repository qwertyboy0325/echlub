# ECHLUB-CURSOR-MULTITASK-SETUP-01

## Objective

Add deterministic Cursor custom-agent definitions, a read-only overlap probe, repository rules, documentation, and a smoke-test skill required to verify async Multitask execution and Grok model routing in a fresh post-reload Cursor chat.

## Authorized file scope

- `.cursor/**`
- `AGENTS.md`
- `docs/ai/cursor-multitask-and-model-routing.md`
- `docs/ai/model-budget.md` (bounded addition only)
- `docs/quality/multitask-smoke-probe.md`
- `docs/work-packages/ECHLUB-CURSOR-MULTITASK-SETUP-01.md`
- `scripts/multitask_probe.py`

## Files created

- `.cursor/agents/echlub-grok-smoke-auditor.md`
- `.cursor/agents/echlub-cargo-smoke-reader.md`
- `.cursor/agents/echlub-package-smoke-reader.md`
- `.cursor/rules/echlub-multitask-governance.mdc`
- `.cursor/skills/echlub-multitask-smoke-probe/SKILL.md`
- `scripts/multitask_probe.py`
- `docs/ai/cursor-multitask-and-model-routing.md`
- `docs/quality/multitask-smoke-probe.md`

## Existing agents normalized

| Agent | readonly | is_background | Notes |
| --- | --- | --- | --- |
| echlub-bootstrap-implementer | false | true | Writable bounded implementer |
| echlub-legacy-archaeologist | true | true | Read-only archaeology |
| echlub-foundation-auditor | true | false | Sequential final auditor |
| echlub-performance-auditor | true | false | Added complete frontmatter; sequential final auditor |

Models preserved for existing agents.

## Setup checks performed

- Zero-wait probe helper smoke for `grok`, `cargo`, and `package` modes
- Frontmatter completeness on all `.cursor/agents/*.md`
- `scripts/verify.py`
- Reference cleanliness

## Runtime probe status

- No actual Multitask invocation in setup chat
- No Grok invocation in setup chat
- `docs/quality/multitask-smoke-probe.md` remains `NOT RUN`

## Required next action

1. Run **Developer: Reload Window**
2. Open a completely new Cursor chat
3. Start from the Agents Window
4. Activate `/multitask` or Plan → Build in Parallel
5. Invoke the `echlub-multitask-smoke-probe` skill

## Stop gate

`LOCAL_MULTITASK_ROUTING_SETUP_COMMITTED`
