# ECHLUB-LIVE-TWO-PEER-OBSERVATION-01

Harden live WebRTC laboratory for two-peer LAN observations with validated endpoint/pair evidence.

## Status

Implementation branch: `work/live-two-peer-observation-01`

Stop gate without owner manual run: `OWNER_LIVE_RUN_REQUIRED` (physical evidence only; not required for draft PR merge review)

```yaml
physical_two_device_test:
  status: deferred
  required_for_this_merge: false
  evidence_claimed: false
  future_gate: separate owner-authorized evidence run
```

## Scope

- Perfect-negotiation live session engine
- `echlub.performance.live-observation/v1` schema and pair validator
- Control-plane LAN bind and strict origin validation
- Live UI, launcher, runbook

## Forbidden

- Transport selection, acoustic latency claims, SDP/ICE/device storage in evidence
