# RTP Reconnect and Harness Final Closure 12 — Writer Self-Review

Work package: `ECHLUB-RTP-RECONNECT-HARNESS-FINAL-CLOSURE-12`

## Explicit search map

| Search term | Result |
| --- | --- |
| cancel preserving available state | **Fixed.** `RtpStatsPreflightController.invalidate()` always sets `cancelled` and clears RTP observations; no `available` preservation branch remains. |
| cancelled controller that can never restart | **Fixed.** Session `ensureStatsPreflight()` calls `restartWhenIdle()` for `cancelled`/`idle` when prerequisites recover. |
| old generation RTP reused after reconnect | **Fixed.** `invalidate()` and `beginGeneration()` reset attempt counters and inbound/outbound seen flags before a new generation starts. |
| overlapping getStats after cancel | **Fixed.** `inFlight`/`inFlightGeneration` gate blocks a second call; `restartWhenIdle()` queues until the pending promise `finally` clears in-flight state. |
| Ready without peerPresent | **Fixed.** `canStartStatsPreflight()` and `collectReadyFailures()` require `peerPresent`; `onPeerLeft` invalidates preflight. |
| DataChannel error leaving availability valid | **Fixed.** `onclosing`, `onclose`, and `onerror` call `invalidateStatsPreflight()`. |
| Samples >= 1 in pre-observation harness guard | **Removed.** Harness uses `evaluatePeerReadyGuardFailures()` with `rtpPreflight.state === available` and allows `Samples: 0`. |
| HARNESS_LIMITATION requiring zero RTP in both directions | **Fixed.** `classifyPeerRtpHarnessLimitation()` accepts any missing required RTP-audio direction with direction-specific reasons. |
| literal `$(shasum ...)` in PR body | **Updated** on PR #1 after final-headed rerun checksum is recorded. |

## C12-001 — generation-safe disconnect and restart

```yaml
status: fixed
changed_symbols:
  - RtpStatsPreflightController.invalidate
  - RtpStatsPreflightController.restartWhenIdle
  - RtpStatsPreflightController.finishInFlight
  - LiveWebRtcSession.invalidateStatsPreflight
  - LiveWebRtcSession.ensureStatsPreflight
  - LiveWebRtcSession.canStartStatsPreflight
  - LiveWebRtcSession.collectReadyFailures
direct_tests:
  - invalidates available state on disconnect invalidate
  - restarts from cancelled after reconnect without reusing old RTP observations
  - does not start a second getStats while the first call is still pending
  - ignores stale getStats results after invalidate
  - restarts bounded polling after exhausted when invalidated for new connection generation
  - does not combine inbound from old generation with outbound from new generation
remaining_risk: full headed ICE disconnect/reconnect timing not unit-tested; covered by session wiring and controller generation tests
```

## C12-002 — harness Ready guard

```yaml
status: fixed
changed_symbols:
  - evaluatePeerReadyGuardFailures
  - assertPeerReadyGuards
  - readDiagnostics remoteTrackLive
direct_tests:
  - passes Ready with preflight available and zero observation samples
  - fails Ready when preflight is probing
  - fails Ready when preflight is exhausted
  - fails Ready when remote track is not live
remaining_risk: none identified for guard semantics
```

## C12-003 — bounded fake-capture limitation classification

```yaml
status: fixed
changed_symbols:
  - classifyPeerRtpHarnessLimitation
  - classifyRtpHarnessLimitation
  - PeerDiagnostics.remoteTrackLive
direct_tests:
  - neither direction seen → HARNESS_LIMITATION
  - outbound only → HARNESS_LIMITATION
  - inbound only → HARNESS_LIMITATION
  - both directions seen but Ready failed elsewhere → not classified
  - remote track not live → not classified
  - connection not connected → not classified
  - bounded window incomplete → not classified
remaining_risk: headed E2E must still prove both peers complete the bounded window while connected
```

## C12-004 — PR and documentation integrity

```yaml
status: fixed
notes:
  - PR #1 report checksum literal replaced with 118d1da113f16da79c875d3865b07379fa41eafb2a350ca50b101783ef02ecba
  - final-head automation @ 80307dc classified HARNESS_LIMITATION with bilateral fake-capture RTP gap
```
