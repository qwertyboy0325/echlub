# Performance Evidence Contract

## Schema

`PerformanceRunV1` — canonical JSON schema validated by `echlub-performance` crate.

## Evidence levels (allowed)

| Level | Description |
| --- | --- |
| `HarnessValidation` | Harness self-check without browser media path |
| `BrowserSyntheticMediaPath` | AudioContext pulse through RTCPeerConnection loopback |
| `BrowserNetworkObservation` | Live two-peer session network metrics |

## Rejected

- `PhysicalAcousticMouthToEar` — explicitly forbidden

## Evidence status

All artifacts: `exploratory_non_authoritative`

## Metric types

```json
{ "kind": "observed", "value": 42.0 }
{ "kind": "unsupported" }
{ "kind": "unavailable", "reason": "..." }
{ "kind": "invalid", "reason": "..." }
```

Never zero-fill missing metrics.

## Privacy exclusions

Forbidden in evidence artifacts:

- IP addresses
- SDP strings
- ICE candidate strings
- deviceId, groupId, media device labels

## Derived metrics

Computed in Rust (`echlub-performance::derived`) only. TypeScript UI exports raw observations; derived metrics are not authoritative in the browser.

## Disclaimer

Every `report.md` must include:

> Exploratory non-authoritative performance observation. Not an acoustic mouth-to-ear measurement. Not a transport selection result.
