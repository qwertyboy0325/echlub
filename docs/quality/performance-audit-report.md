# Performance Baseline Audit Report

> **Supersession note (2026-07-21):** This report reflects ECHLUB-PERFORMANCE-BASELINE-01 only. Re-audit ECHLUB-PERFORMANCE-LAB-REAUDIT-CORRECTION-01 found CORRECTIONS REQUIRED — see `performance-lab-reaudit-report.md` and `performance-baseline-reassessment.md`. Original synthetic artifacts reclassified as FAILED_OBSERVATION.

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
