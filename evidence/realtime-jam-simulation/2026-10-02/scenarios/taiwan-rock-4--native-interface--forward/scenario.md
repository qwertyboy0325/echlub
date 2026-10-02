# Jam scenario: taiwan-rock-4

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `native-interface`  
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
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 20.3 | 20.3 | 22.1 | 20/20 |
| drums@taipei/fiber-wired → guitar@tainan/cable-wired | 33.7 | 33.7 | 35.4 | 20/20 |
| drums@taipei/fiber-wired → vocals@hualien/wifi | 44.3 | 44.3 | 45.9 | 20/20 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 20.3 | 20.3 | 22.1 | 20/20 |
| bass@taichung/fiber-wired → guitar@tainan/cable-wired | 28.3 | 28.3 | 30.9 | 20/20 |
| bass@taichung/fiber-wired → vocals@hualien/wifi | 39.0 | 39.0 | 44.1 | 19/20 |
| guitar@tainan/cable-wired → drums@taipei/fiber-wired | 31.0 | 31.0 | 35.4 | 20/20 |
| guitar@tainan/cable-wired → bass@taichung/fiber-wired | 28.3 | 28.3 | 30.9 | 20/20 |
| guitar@tainan/cable-wired → vocals@hualien/wifi | 52.3 | 52.3 | 54.7 | 20/20 |
| vocals@hualien/wifi → drums@taipei/fiber-wired | 47.0 | 47.0 | 45.9 | 19/19 |
| vocals@hualien/wifi → bass@taichung/fiber-wired | 47.0 | 47.0 | 44.1 | 19/19 |
| vocals@hualien/wifi → guitar@tainan/cable-wired | 55.0 | 55.0 | 54.7 | 19/19 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -2.39% |
| Mean worst heard RMS asynchrony | 51.7 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
