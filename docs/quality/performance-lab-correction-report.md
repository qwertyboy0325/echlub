# Performance Laboratory External Review Correction Report

Package: ECHLUB-PERFORMANCE-LAB-EXTERNAL-REVIEW-CORRECTION-02

## Verdict: PARTIAL

Stop gate: `SYNTHETIC_DECODED_MEDIA_LIMITATION` until decoded-pulse detection passes in harness; live corrections and validators land for owner re-review.

## Owner external review findings mapped

| Finding | Correction |
| --- | --- |
| RTP byte progression used as pulse detection | Removed; bytes only for `media_path_live` |
| Headless synthetic PASS not defensible | `HARNESS_LIMITATION` when decoded media unavailable |
| DataChannel RTT not ping/pong | A sends ping; B pong; A records RTT after pong |
| Hard-coded software commit | Build-time git commit injection |
| Ready gate UI-only | Enforced in `LiveWebRtcSession.startObservation()` |
| Draft/finalized export mixed | Separate draft vs finalized export paths |
| Bilateral clock probes incomplete | Both roles initiate; role namespaced sequences |
| Derived metrics double-count loopback | `endToEndSyntheticMs = setup + loopback` |
| Structural vs observation validity conflated | validate structural; assess-synthetic observation |
| Runner optional success semantics | Fail-closed runner |
| Corrected e419865 evidence overclaimed | Reclassified superseded (RTP bytes) |

## C1 — Synthetic harness

| Item | Status |
| --- | --- |
| Decoded media pulse detector (frequency/envelope/sequence) | Done |
| Inbound bytes path-liveness only | Done |
| HARNESS_LIMITATION classification | Done |
| Actual observation window timestamps | Done |
| Consistent run ID | Done |
| Ping/pong RTT | Done |
| Independent detector tests | Done |

Root cause retained: headless Chromium may deliver inbound RTP without exposing decoded samples to analysers.

## C2 — Live WebRTC

| Item | Status |
| --- | --- |
| Exact commit injection | Done |
| Session ready gate in startObservation | Done |
| Bilateral clock probes | Done |
| Typed stats + candidate categories | Done |
| Cancellable stats sampling | Done |
| Behavioral tests | Done |

## C3 — Rust validators

| Item | Status |
| --- | --- |
| Structural vs observation separation | Done |
| Assessment thresholds (≥5 emit, ≥4 detect, ≥0.8 rate) | Done |
| Corrected derived metrics | Done |
| validate-live-endpoint finalized | Done |
| validate-live-draft command | Done |
| Zero-detection structural fixture | Done |

## C4 — Runner / CI / docs

| Item | Status |
| --- | --- |
| Fail-closed runner | Done |
| Superseded corrected evidence reclassification | Done |
| Active evidence only on decoded pulse PASS | Done |
| Live runbook suspended | Verified |

## Corrected synthetic gate

Active corrected PASS requires decoded-pulse detection meeting assessment thresholds. Prior corrected run at e419865 head reclassified; historical JSON unchanged.

## Model accounting

- requested: composer-2.5-fast
- actual: composer-2.5-fast
- api_quota_used: false
- grok_invoked: false
