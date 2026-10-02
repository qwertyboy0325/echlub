# Bot band over UDP: 4 bots, mode probe, impairment assumed-taiwan

> Real sockets, synthetic musicians, impairment injected in-process. Localhost results describe this machine only.

| Bot | Sent | Uplink drops | Mixes recv | Downlink drops | Concealed | Late ticks |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 2996 | 3 | 2999 | 0 | 1 | 4 |
| 1 | 2998 | 1 | 2999 | 1 | 3 | 0 |
| 2 | 2996 | 3 | 2999 | 2 | 2 | 1 |
| 3 | 2989 | 10 | 2999 | 14 | 18 | 2 |

## Probe latency (capture → playout, no device buffers)

| From → To | n | p50 (ms) | p95 (ms) | max (ms) |
| --- | --- | --- | --- | --- |
| 1 → 0 | 8 | 18.7 | 18.7 | 18.7 |
| 2 → 0 | 8 | 24.0 | 24.0 | 24.0 |
| 3 → 0 | 5 | 26.7 | 26.7 | 26.7 |
| 0 → 1 | 8 | 18.7 | 18.7 | 18.7 |
| 2 → 1 | 8 | 21.3 | 21.3 | 21.3 |
| 3 → 1 | 5 | 24.0 | 24.0 | 24.0 |
| 0 → 2 | 8 | 29.3 | 29.3 | 29.3 |
| 1 → 2 | 8 | 26.7 | 26.7 | 26.7 |
| 3 → 2 | 5 | 34.7 | 34.7 | 34.7 |
| 0 → 3 | 8 | 42.7 | 43.0 | 43.0 |
| 1 → 3 | 8 | 40.0 | 40.0 | 40.0 |
| 2 → 3 | 8 | 45.3 | 45.3 | 45.3 |

Relay: 3018 ticks, 3 late, 11976 packets in, 11996 out, 0 decode errors.
