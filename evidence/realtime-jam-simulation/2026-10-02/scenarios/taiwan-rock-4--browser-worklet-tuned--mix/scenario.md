# Jam scenario: taiwan-rock-4

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `browser-worklet-tuned`  
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
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 39.7 | 39.7 | 48.1 | 20/20 | 0.40% | 90.0 |
| drums@taipei/fiber-wired → guitar@tainan/cable-wired | 55.7 | 55.7 | 61.4 | 20/20 | 0.17% | 39.0 |
| drums@taipei/fiber-wired → vocals@hualien/wifi | 66.3 | 66.3 | 71.9 | 20/20 | 0.63% | 141.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 45.0 | 45.0 | 48.1 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@tainan/cable-wired | 53.0 | 53.0 | 56.9 | 20/20 | 0.12% | 27.0 |
| bass@taichung/fiber-wired → vocals@hualien/wifi | 63.7 | 63.7 | 67.4 | 20/20 | 0.57% | 129.0 |
| guitar@tainan/cable-wired → drums@taipei/fiber-wired | 58.3 | 58.3 | 61.4 | 20/20 | 0.19% | 42.0 |
| guitar@tainan/cable-wired → bass@taichung/fiber-wired | 50.3 | 50.3 | 56.9 | 20/20 | 0.45% | 102.0 |
| guitar@tainan/cable-wired → vocals@hualien/wifi | 77.0 | 77.0 | 80.7 | 19/20 | 0.68% | 153.0 |
| vocals@hualien/wifi → drums@taipei/fiber-wired | 69.0 | 69.0 | 71.9 | 19/19 | 0.73% | 165.0 |
| vocals@hualien/wifi → bass@taichung/fiber-wired | 61.0 | 61.0 | 67.4 | 19/19 | 1.00% | 225.0 |
| vocals@hualien/wifi → guitar@tainan/cable-wired | 77.0 | 77.0 | 80.7 | 19/19 | 0.77% | 174.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -4.60% |
| Mean worst heard RMS asynchrony | 82.7 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
