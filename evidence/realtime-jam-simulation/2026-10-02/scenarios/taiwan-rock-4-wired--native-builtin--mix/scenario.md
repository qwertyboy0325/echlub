# Jam scenario: taiwan-rock-4-wired

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
| 2 | guitar@tainan/fiber-wired | 3 | 3.4 |
| 3 | vocals@kaohsiung/fiber-wired | 3 | 3.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 29.3 | 29.3 | 37.7 | 20/20 | 0.40% | 90.0 |
| drums@taipei/fiber-wired → guitar@tainan/fiber-wired | 37.3 | 37.3 | 41.5 | 20/20 | 0.13% | 30.0 |
| drums@taipei/fiber-wired → vocals@kaohsiung/fiber-wired | 34.7 | 34.7 | 42.0 | 20/20 | 0.16% | 36.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 34.7 | 34.7 | 37.7 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@tainan/fiber-wired | 34.7 | 34.7 | 37.0 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → vocals@kaohsiung/fiber-wired | 32.0 | 32.0 | 37.5 | 20/20 | 0.11% | 24.0 |
| guitar@tainan/fiber-wired → drums@taipei/fiber-wired | 40.0 | 40.0 | 41.5 | 20/20 | 0.09% | 21.0 |
| guitar@tainan/fiber-wired → bass@taichung/fiber-wired | 32.0 | 32.0 | 37.0 | 20/20 | 0.36% | 81.0 |
| guitar@tainan/fiber-wired → vocals@kaohsiung/fiber-wired | 37.3 | 37.3 | 41.3 | 20/20 | 0.12% | 27.0 |
| vocals@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 37.3 | 37.3 | 42.0 | 19/19 | 0.09% | 21.0 |
| vocals@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 29.3 | 29.3 | 37.5 | 19/19 | 0.36% | 81.0 |
| vocals@kaohsiung/fiber-wired → guitar@tainan/fiber-wired | 37.3 | 37.3 | 41.3 | 19/19 | 0.09% | 21.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -2.61% |
| Mean worst heard RMS asynchrony | 48.4 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
