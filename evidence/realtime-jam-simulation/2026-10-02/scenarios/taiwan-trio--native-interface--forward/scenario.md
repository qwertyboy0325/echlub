# Jam scenario: taiwan-trio

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
| 2 | guitar@kaohsiung/cable-wired | 5 | 8.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 20.3 | 20.3 | 22.1 | 20/20 | 0.15% | 33.0 |
| drums@taipei/fiber-wired → guitar@kaohsiung/cable-wired | 31.0 | 31.0 | 35.9 | 20/20 | 0.16% | 36.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 20.3 | 20.3 | 22.1 | 20/20 | 0.07% | 15.0 |
| bass@taichung/fiber-wired → guitar@kaohsiung/cable-wired | 28.3 | 28.3 | 31.4 | 20/20 | 0.15% | 33.0 |
| guitar@kaohsiung/cable-wired → drums@taipei/fiber-wired | 33.7 | 33.7 | 35.9 | 20/20 | 0.24% | 54.0 |
| guitar@kaohsiung/cable-wired → bass@taichung/fiber-wired | 31.0 | 31.0 | 31.4 | 20/20 | 0.15% | 33.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -1.76% |
| Mean worst heard RMS asynchrony | 38.1 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
