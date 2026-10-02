# Jam scenario: taiwan-rock-4

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `browser-worklet-tuned`  
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

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 39.7 | 39.7 | 41.4 | 20/20 | 0.15% | 33.0 |
| drums@taipei/fiber-wired → guitar@tainan/cable-wired | 53.0 | 53.0 | 54.7 | 20/20 | 0.19% | 42.0 |
| drums@taipei/fiber-wired → vocals@hualien/wifi | 63.7 | 63.7 | 65.2 | 20/20 | 0.67% | 150.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 39.7 | 39.7 | 41.4 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@tainan/cable-wired | 47.7 | 47.7 | 50.2 | 20/20 | 0.15% | 33.0 |
| bass@taichung/fiber-wired → vocals@hualien/wifi | 58.3 | 58.3 | 63.4 | 19/20 | 0.52% | 117.0 |
| guitar@tainan/cable-wired → drums@taipei/fiber-wired | 50.3 | 50.3 | 54.7 | 20/20 | 0.24% | 54.0 |
| guitar@tainan/cable-wired → bass@taichung/fiber-wired | 47.7 | 47.7 | 50.2 | 20/20 | 0.15% | 33.0 |
| guitar@tainan/cable-wired → vocals@hualien/wifi | 71.7 | 71.7 | 74.0 | 20/20 | 0.64% | 144.0 |
| vocals@hualien/wifi → drums@taipei/fiber-wired | 66.3 | 66.3 | 65.2 | 19/19 | 0.60% | 135.0 |
| vocals@hualien/wifi → bass@taichung/fiber-wired | 66.3 | 66.3 | 63.4 | 19/19 | 0.63% | 141.0 |
| vocals@hualien/wifi → guitar@tainan/cable-wired | 74.3 | 74.3 | 74.0 | 19/19 | 0.60% | 135.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -4.28% |
| Mean worst heard RMS asynchrony | 77.9 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
