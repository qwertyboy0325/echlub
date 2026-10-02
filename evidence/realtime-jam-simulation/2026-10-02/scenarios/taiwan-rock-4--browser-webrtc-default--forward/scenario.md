# Jam scenario: taiwan-rock-4

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `browser-webrtc-default`  
- Topology: Forward  
- Cross-ISP penalty: false  
- Tempo: 120 BPM, 32 bars  
- Ensemble runs: 20

## Players

| # | Player | Jitter depth (frames) | Link expected one-way (ms) |
| --- | --- | --- | --- |
| 0 | drums@taipei/fiber-wired | 3 | 4.1 |
| 1 | bass@taichung/fiber-wired | 2 | 2.3 |
| 2 | guitar@tainan/cable-wired | 5 | 7.6 |
| 3 | vocals@hualien/wifi | 8 | 10.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected |
| --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 110.6 | 110.6 | 112.4 | 20/20 |
| drums@taipei/fiber-wired → guitar@tainan/cable-wired | 116.0 | 116.0 | 117.7 | 20/20 |
| drums@taipei/fiber-wired → vocals@hualien/wifi | 118.6 | 118.6 | 120.2 | 20/20 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 108.0 | 108.0 | 109.7 | 20/20 |
| bass@taichung/fiber-wired → guitar@tainan/cable-wired | 110.6 | 110.6 | 113.2 | 20/20 |
| bass@taichung/fiber-wired → vocals@hualien/wifi | 113.3 | 113.3 | 118.4 | 19/20 |
| guitar@tainan/cable-wired → drums@taipei/fiber-wired | 118.6 | 118.6 | 123.0 | 20/20 |
| guitar@tainan/cable-wired → bass@taichung/fiber-wired | 118.6 | 118.6 | 121.2 | 20/20 |
| guitar@tainan/cable-wired → vocals@hualien/wifi | 126.6 | 126.6 | 129.0 | 20/20 |
| vocals@hualien/wifi → drums@taipei/fiber-wired | 134.6 | 134.6 | 133.5 | 19/19 |
| vocals@hualien/wifi → bass@taichung/fiber-wired | 137.3 | 137.3 | 134.4 | 19/19 |
| vocals@hualien/wifi → guitar@tainan/cable-wired | 137.3 | 137.3 | 137.0 | 19/19 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -10.20% |
| Mean worst heard RMS asynchrony | 170.3 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
