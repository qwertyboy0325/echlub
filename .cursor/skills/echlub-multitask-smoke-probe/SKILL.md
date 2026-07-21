# EchLub Multitask Smoke Probe

Use this skill only in a **fresh Cursor chat** after reloading the window following agent-catalog changes.

Canonical policy: `docs/ai/cursor-multitask-and-model-routing.md`
Result template: `docs/quality/multitask-smoke-probe.md`

## Preconditions

1. `ECHLUB-CURSOR-MULTITASK-SETUP-01` is committed.
2. Cursor window has been reloaded.
3. Multitask is activated from the Agents Window via `/multitask` or Plan → Build in Parallel.
4. Repository worktree is clean.

## Launch surface

Record whether the parent used `/multitask` or Plan → Build in Parallel. Prompt wording alone is not activation proof.

## Required dispatch

Launch exactly these three custom subagents **before awaiting any of them**:

1. `echlub-grok-smoke-auditor` — run `python3 scripts/multitask_probe.py --mode grok --sleep-seconds 15`
2. `echlub-cargo-smoke-reader` — run `python3 scripts/multitask_probe.py --mode cargo --sleep-seconds 20`
3. `echlub-package-smoke-reader` — run `python3 scripts/multitask_probe.py --mode package --sleep-seconds 20`

Forbidden during probe:

- repository writes;
- model fallback;
- invoking agents created in the same chat before reload;
- awaiting one child before dispatching the next independent child.

## Concurrency rule

Two intervals overlap when:

```text
max(start_a, start_b) < min(end_a, end_b)
```

Use each child’s reported `start_timestamp` and `end_timestamp`, or script JSON monotonic bounds when timestamps are unavailable.

## PASS criteria

```text
child_agent_count = 3
all_dispatched_before_wait = true
at_least_two_intervals_overlap = true
custom_grok_agent_discovered = true
actual_model = cursor-grok-4.5-high-fast
model_identity_verifiable = true
repository_modified = false
```

Missing runtime model evidence must result in `MODEL_IDENTITY_UNVERIFIABLE`, not PASS.

## Recording results

Update `docs/quality/multitask-smoke-probe.md` only after the probe completes. Do not mark PASS during setup packages.

Each child must report separately:

- requested_model
- actual_model
- model_identity_source
- model_identity_verifiable
