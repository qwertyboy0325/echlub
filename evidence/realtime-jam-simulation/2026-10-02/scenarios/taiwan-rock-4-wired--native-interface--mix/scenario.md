# Jam scenario: taiwan-rock-4-wired

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
| 2 | guitar@tainan/fiber-wired | 3 | 3.4 |
| 3 | vocals@kaohsiung/fiber-wired | 3 | 3.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 20.3 | 20.3 | 28.7 | 20/20 | 0.40% | 90.0 |
| drums@taipei/fiber-wired → guitar@tainan/fiber-wired | 28.3 | 28.3 | 32.5 | 20/20 | 0.13% | 30.0 |
| drums@taipei/fiber-wired → vocals@kaohsiung/fiber-wired | 25.7 | 25.7 | 33.0 | 20/20 | 0.16% | 36.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 25.7 | 25.7 | 28.7 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@tainan/fiber-wired | 25.7 | 25.7 | 28.0 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → vocals@kaohsiung/fiber-wired | 23.0 | 23.0 | 28.5 | 20/20 | 0.11% | 24.0 |
| guitar@tainan/fiber-wired → drums@taipei/fiber-wired | 31.0 | 31.0 | 32.5 | 20/20 | 0.09% | 21.0 |
| guitar@tainan/fiber-wired → bass@taichung/fiber-wired | 23.0 | 23.0 | 28.0 | 20/20 | 0.36% | 81.0 |
| guitar@tainan/fiber-wired → vocals@kaohsiung/fiber-wired | 28.3 | 28.3 | 32.3 | 20/20 | 0.12% | 27.0 |
| vocals@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 28.3 | 28.3 | 33.0 | 19/19 | 0.09% | 21.0 |
| vocals@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 20.3 | 20.3 | 28.5 | 19/19 | 0.36% | 81.0 |
| vocals@kaohsiung/fiber-wired → guitar@tainan/fiber-wired | 28.3 | 28.3 | 32.3 | 19/19 | 0.09% | 21.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -1.71% |
| Mean worst heard RMS asynchrony | 36.5 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
