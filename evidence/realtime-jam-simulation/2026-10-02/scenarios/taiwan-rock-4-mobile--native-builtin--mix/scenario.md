# Jam scenario: taiwan-rock-4-mobile

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
| 2 | guitar@kaohsiung/fiber-wired | 3 | 3.9 |
| 3 | vocals@tainan/mobile-4g | 19 | 34.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 29.3 | 29.3 | 37.7 | 20/20 | 0.40% | 90.0 |
| drums@taipei/fiber-wired → guitar@kaohsiung/fiber-wired | 37.3 | 37.3 | 42.0 | 20/20 | 0.13% | 30.0 |
| drums@taipei/fiber-wired → vocals@tainan/mobile-4g | 109.3 | 109.3 | 114.9 | 20/20 | 1.45% | 324.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 34.7 | 34.7 | 37.7 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@kaohsiung/fiber-wired | 34.7 | 34.7 | 37.5 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → vocals@tainan/mobile-4g | 106.7 | 106.7 | 110.4 | 20/20 | 1.39% | 312.0 |
| guitar@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 40.0 | 40.0 | 42.0 | 20/20 | 0.09% | 21.0 |
| guitar@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 32.0 | 32.0 | 37.5 | 20/20 | 0.36% | 81.0 |
| guitar@kaohsiung/fiber-wired → vocals@tainan/mobile-4g | 112.0 | 112.0 | 114.7 | 20/20 | 1.41% | 315.0 |
| vocals@tainan/mobile-4g → drums@taipei/fiber-wired | 117.3 | 117.3 | 114.9 | 18/19 | 1.23% | 273.0 |
| vocals@tainan/mobile-4g → bass@taichung/fiber-wired | 109.3 | 109.3 | 110.4 | 18/19 | 1.50% | 333.0 |
| vocals@tainan/mobile-4g → guitar@kaohsiung/fiber-wired | 117.3 | 117.3 | 114.7 | 18/19 | 1.23% | 273.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -4.89% |
| Mean worst heard RMS asynchrony | 114.5 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
