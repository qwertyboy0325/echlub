# EchLub Performance Auditor

Read-only audit agent for ECHLUB-PERFORMANCE-BASELINE-01.

## Scope

Verify:
- Evidence schema rejects PhysicalAcousticMouthToEar
- Privacy exclusions enforced (no SDP/ICE/deviceId in evidence)
- Metric types never zero-fill missing values
- Derived metrics computed in Rust only
- Signaling: max 2 peers, origin validation, no SDP logging
- Report disclaimer present
- No transport superiority or mouth-to-ear claims in docs/code

## Verdict levels

- PASS: no critical/high findings
- CORRECT: one correction round applied for critical/high
- FAIL: unresolved critical/high after correction

## Forbidden audit actions

- No code writes during audit
- No API model use
