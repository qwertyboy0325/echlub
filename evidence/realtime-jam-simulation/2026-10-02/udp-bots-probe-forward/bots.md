# Bot band over UDP: 4 bots, mode probe, impairment assumed-taiwan

> Real sockets, synthetic musicians, impairment injected in-process. Localhost results describe this machine only.

| Bot | Sent | Uplink drops | Mixes recv | Downlink drops | Concealed | Late ticks |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 2996 | 3 | 8974 | 3 | 23 | 2 |
| 1 | 2998 | 1 | 8971 | 4 | 27 | 2 |
| 2 | 2996 | 3 | 8976 | 12 | 29 | 3 |
| 3 | 2989 | 10 | 8983 | 41 | 52 | 1 |

## Probe latency (capture → playout, no device buffers)

| From → To | n | p50 (ms) | p95 (ms) | max (ms) |
| --- | --- | --- | --- | --- |
| 1 → 0 | 8 | 16.0 | 16.0 | 16.0 |
| 2 → 0 | 8 | 29.3 | 29.3 | 29.3 |
| 3 → 0 | 8 | 45.3 | 45.3 | 45.3 |
| 0 → 1 | 8 | 16.0 | 16.0 | 16.0 |
| 2 → 1 | 8 | 26.7 | 26.7 | 26.7 |
| 3 → 1 | 8 | 42.7 | 42.7 | 42.7 |
| 0 → 2 | 8 | 32.0 | 32.0 | 32.0 |
| 1 → 2 | 8 | 24.0 | 24.0 | 24.0 |
| 3 → 2 | 8 | 53.3 | 53.3 | 53.3 |
| 0 → 3 | 8 | 40.0 | 40.0 | 40.0 |
| 1 → 3 | 8 | 37.3 | 37.3 | 37.3 |
| 2 → 3 | 8 | 45.3 | 45.3 | 45.3 |

Relay: 3018 ticks, 7 late, 11972 packets in, 35904 out, 0 decode errors.
