# Performance Laboratory Re-Audit Report

Audit: ECHLUB-PERFORMANCE-LAB-REAUDIT-CORRECTION-01 (read-only)

## Verdict: CORRECTIONS REQUIRED

Prior audit PASS (ECHLUB-PERFORMANCE-BASELINE-01) is superseded. Structural validation passed but observation quality gates were absent.

## A1 — Synthetic harness, historical evidence, timing semantics

| Finding | Severity | Detail |
| --- | --- | --- |
| Zero-detection artifacts marked VALID | Critical | Committed `synthetic-2026-07-21T08-29-*.json` have `pulsesDetected: 0`, `bytesReceived: 0` |
| No assess vs validate separation | High | `validate` is schema/privacy-only; runner ignored validate failures (exit 0) |
| Clock domain mismatch | High | Emit uses `performance.now()`, detect uses AudioWorklet `currentTime * 1000` |
| No pulse correlation | High | Naive index pairing; no sequence IDs or ontrack wait |
| Zero-fill guard broken | High | `check_zero_filled_metrics` matches path substring `"Unavailable"` only |
| Failed metrics exported as `observed: 0` | High | Violates evidence contract ("never zero-fill missing metrics") |

## A2 — Live WebRTC runtime

| Finding | Severity | Detail |
| --- | --- | --- |
| Mic attach after connect | High | `replaceTrack` requires transceiver created in `connect()`; UI enables mic first |
| Ready gate bypass | Medium | UI allows observation from `connected` without clock probes |
| Clock probes deferred | Medium | Probes start in `startObservation()`, blocking `ready_to_observe` pre-observation |
| Stats sampling overlap | Low | Async `getStats` ticks may overlap without serialization |

Negotiation (perfect negotiation), privacy exclusions, and typed stats normalization are structurally present.

## A3 — Rust validators, pairing, CLI

| Finding | Severity | Detail |
| --- | --- | --- |
| No `assess_synthetic_observation` | Critical | Quality assessment command missing |
| Derived metrics on empty path | Medium | `end_to_end_synthetic_ms` computed with zero detections → negative values |
| Live finalized validation | OK | `require_finalized` enforced at pair time |
| Pairing + checksums | OK | Present in `live_pair.rs` |

## A4 — Tests, runners, CI, claims

| Finding | Severity | Detail |
| --- | --- | --- |
| No negative zero-detection fixture | High | CI cannot reject failed media path |
| Browser harness not in CI | Medium | Expected; synthetic rerun is manual/optional |
| Prior PASS claim overstated | High | "Synthetic evidence validated" implied observation validity |

## Required corrections

See ECHLUB-PERFORMANCE-LAB-REAUDIT-CORRECTION-01 sections B1–B4.

## Model accounting

- requested: composer-2.5-fast
- actual: composer-2.5-fast
- api_quota_used: false
- grok_invoked: false
