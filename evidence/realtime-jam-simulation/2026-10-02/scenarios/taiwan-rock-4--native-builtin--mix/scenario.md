# Jam scenario: taiwan-rock-4

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `native-builtin`  
- Topology: Mix  
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
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 29.3 | 29.3 | 37.7 | 20/20 |
| drums@taipei/fiber-wired → guitar@tainan/cable-wired | 45.3 | 45.3 | 51.0 | 20/20 |
| drums@taipei/fiber-wired → vocals@hualien/wifi | 56.0 | 56.0 | 61.5 | 20/20 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 34.7 | 34.7 | 37.7 | 20/20 |
| bass@taichung/fiber-wired → guitar@tainan/cable-wired | 42.7 | 42.7 | 46.6 | 20/20 |
| bass@taichung/fiber-wired → vocals@hualien/wifi | 53.3 | 53.3 | 57.1 | 20/20 |
| guitar@tainan/cable-wired → drums@taipei/fiber-wired | 48.0 | 48.0 | 51.0 | 20/20 |
| guitar@tainan/cable-wired → bass@taichung/fiber-wired | 40.0 | 40.0 | 46.6 | 20/20 |
| guitar@tainan/cable-wired → vocals@hualien/wifi | 66.7 | 66.7 | 70.4 | 19/20 |
| vocals@hualien/wifi → drums@taipei/fiber-wired | 58.7 | 58.7 | 61.5 | 19/19 |
| vocals@hualien/wifi → bass@taichung/fiber-wired | 50.7 | 50.7 | 57.1 | 19/19 |
| vocals@hualien/wifi → guitar@tainan/cable-wired | 66.7 | 66.7 | 70.4 | 19/19 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -3.61% |
| Mean worst heard RMS asynchrony | 68.3 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
