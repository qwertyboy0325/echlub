# Final Restart Race and Classification Closure 13 — Writer Self-Review

Work package: `ECHLUB-FINAL-RESTART-RACE-CLASSIFICATION-CLOSURE-13`

## C13-001 — invalidate queued restart requests

```yaml
status: fixed
changed_symbols:
  - RtpStatsPreflightController.restartRequestedForGeneration
  - RtpStatsPreflightController.invalidate
  - RtpStatsPreflightController.restartWhenIdle
  - RtpStatsPreflightController.finishInFlight
direct_tests:
  - clears queued restart when a second invalidate supersedes reconnect
  - does not auto-restart after available invalidate when reconnect is superseded
remaining_risk: none identified for generation-bound restart queue
```

## C13-002 — fail-closed HARNESS_LIMITATION classification

```yaml
status: fixed
changed_symbols:
  - mayClassifyScenarioAsHarnessLimitation
  - runScenario finally ordering
direct_tests:
  - fail-closed HARNESS_LIMITATION gate suite in harness.test.ts
remaining_risk: headed E2E must still satisfy all gate conditions under real browser teardown
```

## C13-003 — result packet consistency

```yaml
status: fixed
notes:
  - manifest_checksums omitted from final result packet when no endpoints exported
  - report_checksum retained separately
```
