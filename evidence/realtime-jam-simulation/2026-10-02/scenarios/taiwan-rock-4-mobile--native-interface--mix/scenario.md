# Jam scenario: taiwan-rock-4-mobile

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
| 2 | guitar@kaohsiung/fiber-wired | 3 | 3.9 |
| 3 | vocals@tainan/mobile-4g | 19 | 34.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 20.3 | 20.3 | 28.7 | 20/20 | 0.40% | 90.0 |
| drums@taipei/fiber-wired → guitar@kaohsiung/fiber-wired | 28.3 | 28.3 | 33.0 | 20/20 | 0.13% | 30.0 |
| drums@taipei/fiber-wired → vocals@tainan/mobile-4g | 100.3 | 100.3 | 105.9 | 20/20 | 1.45% | 324.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 25.7 | 25.7 | 28.7 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@kaohsiung/fiber-wired | 25.7 | 25.7 | 28.5 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → vocals@tainan/mobile-4g | 97.7 | 97.7 | 101.4 | 20/20 | 1.39% | 312.0 |
| guitar@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 31.0 | 31.0 | 33.0 | 20/20 | 0.09% | 21.0 |
| guitar@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 23.0 | 23.0 | 28.5 | 20/20 | 0.36% | 81.0 |
| guitar@kaohsiung/fiber-wired → vocals@tainan/mobile-4g | 103.0 | 103.0 | 105.7 | 20/20 | 1.41% | 315.0 |
| vocals@tainan/mobile-4g → drums@taipei/fiber-wired | 108.3 | 108.3 | 105.9 | 18/19 | 1.23% | 273.0 |
| vocals@tainan/mobile-4g → bass@taichung/fiber-wired | 100.3 | 100.3 | 101.4 | 18/19 | 1.50% | 333.0 |
| vocals@tainan/mobile-4g → guitar@kaohsiung/fiber-wired | 108.3 | 108.3 | 105.7 | 18/19 | 1.23% | 273.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -4.03% |
| Mean worst heard RMS asynchrony | 104.1 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
