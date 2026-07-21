# Cursor Multitask and Model Routing

This document defines how EchLub uses Cursor Agent, subagents, Multitask, background agents, and worktrees.

## Surfaces

| Surface | Role |
| --- | --- |
| Ordinary Agent chat | Parent coordinator or single-threaded execution |
| Subagent / Task tool | Child execution with explicit ownership |
| `/multitask` or Plan → Build in Parallel | External activation required for concurrent child dispatch |
| Background agents (`is_background: true`) | Async child work; parent must not duplicate foreground execution |
| Worktrees | Cursor-managed isolation; subagents must not create manual worktrees |

## Activation is not prompt text

Writing “use Multitask” or “run tasks in parallel” inside an ordinary prompt does **not** prove Multitask activation.

Work packages that require concurrent subagents must declare a launch precondition:

- Start from the Cursor Agents Window using `/multitask`, or
- Create a Plan and choose **Build in Parallel**

The final report must record which surface was used.

## Dispatch-before-wait

Independent subagents must be dispatched before awaiting results:

```text
dispatch A → dispatch B → dispatch C → await all
```

Serial `dispatch A → wait A → dispatch B` is not acceptable concurrency evidence.

## Dependency rule

Only genuinely independent work may run concurrently.

Final package auditors (`is_background: false`) run **after** implementation and evidence completion. They are custom child agents, but they are sequential gates—not parallel peers of the implementation they review.

## Custom-agent catalog lifecycle

After adding or changing `.cursor/agents/**`:

1. commit the changes;
2. reload the Cursor window;
3. open a fresh chat;
4. invoke the custom agent.

Do not create a custom agent and rely on it in the same pre-reload chat.

## Requested model versus actual runtime model

Agent frontmatter `model:` records the **requested** model only.

A Grok routing PASS requires runtime or execution metadata identifying:

```text
actual_model = cursor-grok-4.5-high-fast
model_identity_verifiable = true
```

Agent prose claiming “I am Grok” is not proof.

When runtime identity is unavailable, classify as `MODEL_IDENTITY_UNVERIFIABLE`.

## Failure classifications

```text
MULTITASK_NOT_ACTIVATED
TASK_TOOL_UNAVAILABLE
CUSTOM_AGENT_NOT_DISCOVERED
GROK_MODEL_UNAVAILABLE
GROK_MODEL_MISMATCH
MODEL_IDENTITY_UNVERIFIABLE
BACKGROUND_EXECUTION_BLOCKING
PARENT_CHOSE_SERIAL_EXECUTION
SUBAGENT_START_FAILED
```

## Cost boundary

The Multitask smoke probe may use:

- one Grok custom subagent;
- two Composer smoke readers;
- one parent coordinator.

No API-backed models or automatic premium fallback is authorized.

## Current status

| Capability | Status |
| --- | --- |
| Multitask parallel execution | **Verified** — three custom subagents dispatched before await; concurrency proven |
| Custom-agent discovery | **Verified** — `echlub-grok-smoke-auditor`, cargo/package smoke readers discovered and invoked |
| Grok runtime model routing | **Unverified** — requested `cursor-grok-4.5-high-fast`; actual runtime model not recorded in verifiable metadata |

Probe verdict: `FAIL` with `MODEL_IDENTITY_UNVERIFIABLE`. See `docs/quality/multitask-smoke-probe.md`.

Agent frontmatter `model:` remains requested-model only. Do not treat probe PASS for Grok routing until `model_identity_verifiable: true` and `actual_model = cursor-grok-4.5-high-fast` are recorded from runtime or execution metadata.
