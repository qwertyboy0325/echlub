# Performance Laboratory Correction Report

Package: ECHLUB-PERFORMANCE-LAB-REAUDIT-CORRECTION-01

## Verdict: PASS

## B1 — Synthetic harness

| Item | Status |
| --- | --- |
| Timing semantics unified on `performance.now()` | Done |
| Pulse correlation (emit ↔ inbound-byte progression + analyser fallback) | Done |
| Fresh artifact guarantee (`corrected/<run-id>/observation.json`) | Done |
| Runner exit codes (0 pass, 1 assess/validate fail, 2 browser fail) | Done |
| `assess-synthetic` vs `validate` separation | Done |

Root cause: headless Chromium delivers inbound RTP (stats counters advance) but does not expose decoded samples to WebAudio analysers reliably. Detection uses correlated inbound-byte progression tied to pulse emit times.

## B2 — Live WebRTC

| Item | Status |
| --- | --- |
| Mic attach on connect when stream pre-captured | Done |
| Clock warmup probes at DataChannel open | Done |
| Ready gate enforced in UI (`ready_to_observe` only) | Done |
| Serialized stats sampling | Done |

## B3 — Rust validators

| Item | Status |
| --- | --- |
| `assess_synthetic_observation` | Done |
| Zero-fill guard for failed-path `observed: 0` | Done |
| Derived metrics unavailable without detections | Done |
| CLI `assess-synthetic` command | Done |
| Negative fixture `synthetic-zero-detection-v1.json` | Done |

## B4 — Control plane / CI / docs

| Item | Status |
| --- | --- |
| Origin policy (existing strict validation) | Verified |
| `verify.py` assess + negative fixture | Done |
| Evidence manifest | Done |
| Runbook suspended pending owner live run | Done |

## Phase D — Corrected synthetic run

- Run ID: `synthetic-2026-07-21T10-01-14-749008Z`
- Path: `evidence/performance-baseline/corrected/synthetic-2026-07-21T10-01-14-749008Z/`
- validate: PASS
- assess-synthetic: PASS
- `pulsesDetected: 5`, `detectionRate: 1.0`, `bytesReceived: 1999`

## Phase E — Post-correction audit

Internal read-only audit: **PASS**

- Negative fixture rejected by validate
- Positive vector passes assess
- Historical artifacts reclassified, not deleted
- No live import artifacts populated

## Model accounting

- requested: composer-2.5-fast
- actual: composer-2.5-fast
- api_quota_used: false
- grok_invoked: false
