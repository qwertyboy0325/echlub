# Jam scenario: taiwan-trio

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
| 2 | guitar@kaohsiung/cable-wired | 5 | 8.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected |
| --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 20.3 | 20.3 | 28.7 | 20/20 |
| drums@taipei/fiber-wired → guitar@kaohsiung/cable-wired | 36.3 | 36.3 | 42.5 | 20/20 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 25.7 | 25.7 | 28.7 | 20/20 |
| bass@taichung/fiber-wired → guitar@kaohsiung/cable-wired | 33.7 | 33.7 | 38.1 | 20/20 |
| guitar@kaohsiung/cable-wired → drums@taipei/fiber-wired | 39.0 | 39.0 | 42.5 | 20/20 |
| guitar@kaohsiung/cable-wired → bass@taichung/fiber-wired | 31.0 | 31.0 | 38.1 | 20/20 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -2.11% |
| Mean worst heard RMS asynchrony | 43.6 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
