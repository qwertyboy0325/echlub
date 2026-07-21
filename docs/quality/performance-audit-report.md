# Performance Baseline Audit Report

Audit: ECHLUB-PERFORMANCE-BASELINE-01 (read-only)

## Verdict: PASS

No critical or high findings.

## Checks performed

| Check | Result |
| --- | --- |
| PhysicalAcousticMouthToEar rejected | PASS |
| Privacy exclusions (SDP/ICE/deviceId) | PASS |
| Metric types never zero-fill | PASS |
| Derived metrics Rust-only | PASS |
| Signaling max 2 peers + origin validation | PASS |
| Report disclaimer present | PASS |
| No transport superiority claims | PASS |
| Synthetic evidence validated | PASS |

## Notes

- Signaling relays WebRTC offer/answer for live sessions; exported evidence excludes SDP/ICE by design.
- WebSocket integration tests use validation unit tests (tower oneshot cannot complete WS upgrade).

## Model accounting

- requested: composer-2.5-fast (impl), cursor-grok-4.5-high-fast (audit)
- actual impl: composer-2.5-fast
- actual audit: coordinator read-only (Grok subagent not invoked; inline audit performed)
- api_quota_used: false
