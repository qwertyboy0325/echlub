# Jam scenario: taiwan-rock-4

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `native-interface`  
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

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 20.3 | 20.3 | 28.7 | 20/20 | 0.40% | 90.0 |
| drums@taipei/fiber-wired → guitar@tainan/cable-wired | 36.3 | 36.3 | 42.0 | 20/20 | 0.17% | 39.0 |
| drums@taipei/fiber-wired → vocals@hualien/wifi | 47.0 | 47.0 | 52.5 | 20/20 | 0.63% | 141.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 25.7 | 25.7 | 28.7 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@tainan/cable-wired | 33.7 | 33.7 | 37.6 | 20/20 | 0.12% | 27.0 |
| bass@taichung/fiber-wired → vocals@hualien/wifi | 44.3 | 44.3 | 48.1 | 20/20 | 0.57% | 129.0 |
| guitar@tainan/cable-wired → drums@taipei/fiber-wired | 39.0 | 39.0 | 42.0 | 20/20 | 0.19% | 42.0 |
| guitar@tainan/cable-wired → bass@taichung/fiber-wired | 31.0 | 31.0 | 37.6 | 20/20 | 0.45% | 102.0 |
| guitar@tainan/cable-wired → vocals@hualien/wifi | 57.7 | 57.7 | 61.4 | 19/20 | 0.68% | 153.0 |
| vocals@hualien/wifi → drums@taipei/fiber-wired | 49.7 | 49.7 | 52.5 | 19/19 | 0.73% | 165.0 |
| vocals@hualien/wifi → bass@taichung/fiber-wired | 41.7 | 41.7 | 48.1 | 19/19 | 1.00% | 225.0 |
| vocals@hualien/wifi → guitar@tainan/cable-wired | 57.7 | 57.7 | 61.4 | 19/19 | 0.77% | 174.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -2.73% |
| Mean worst heard RMS asynchrony | 56.2 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
