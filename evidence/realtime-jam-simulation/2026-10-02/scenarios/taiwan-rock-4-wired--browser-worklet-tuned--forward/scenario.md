# Jam scenario: taiwan-rock-4-wired

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
| 2 | guitar@tainan/fiber-wired | 3 | 3.4 |
| 3 | vocals@kaohsiung/fiber-wired | 3 | 3.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected |
| --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 39.7 | 39.7 | 41.4 | 20/20 |
| drums@taipei/fiber-wired → guitar@tainan/fiber-wired | 42.3 | 42.3 | 45.2 | 20/20 |
| drums@taipei/fiber-wired → vocals@kaohsiung/fiber-wired | 42.3 | 42.3 | 45.7 | 20/20 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 39.7 | 39.7 | 41.4 | 20/20 |
| bass@taichung/fiber-wired → guitar@tainan/fiber-wired | 39.7 | 39.7 | 40.7 | 20/20 |
| bass@taichung/fiber-wired → vocals@kaohsiung/fiber-wired | 39.7 | 39.7 | 41.2 | 20/20 |
| guitar@tainan/fiber-wired → drums@taipei/fiber-wired | 42.3 | 42.3 | 45.2 | 20/20 |
| guitar@tainan/fiber-wired → bass@taichung/fiber-wired | 39.7 | 39.7 | 40.7 | 20/20 |
| guitar@tainan/fiber-wired → vocals@kaohsiung/fiber-wired | 42.3 | 42.3 | 45.0 | 20/20 |
| vocals@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 42.3 | 42.3 | 45.7 | 19/19 |
| vocals@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 37.0 | 37.0 | 41.2 | 19/19 |
| vocals@kaohsiung/fiber-wired → guitar@tainan/fiber-wired | 42.3 | 42.3 | 45.0 | 19/19 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -3.22% |
| Mean worst heard RMS asynchrony | 56.7 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
