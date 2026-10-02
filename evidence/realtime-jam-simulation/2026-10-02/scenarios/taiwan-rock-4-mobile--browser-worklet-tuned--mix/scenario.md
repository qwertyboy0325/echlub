# Jam scenario: taiwan-rock-4-mobile

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
| 2 | guitar@kaohsiung/fiber-wired | 3 | 3.9 |
| 3 | vocals@tainan/mobile-4g | 19 | 34.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 39.7 | 39.7 | 48.1 | 20/20 | 0.40% | 90.0 |
| drums@taipei/fiber-wired → guitar@kaohsiung/fiber-wired | 47.7 | 47.7 | 52.3 | 20/20 | 0.13% | 30.0 |
| drums@taipei/fiber-wired → vocals@tainan/mobile-4g | 119.7 | 119.7 | 125.2 | 20/20 | 1.45% | 324.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 45.0 | 45.0 | 48.1 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@kaohsiung/fiber-wired | 45.0 | 45.0 | 47.9 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → vocals@tainan/mobile-4g | 117.0 | 117.0 | 120.7 | 20/20 | 1.39% | 312.0 |
| guitar@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 50.3 | 50.3 | 52.3 | 20/20 | 0.09% | 21.0 |
| guitar@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 42.3 | 42.3 | 47.9 | 20/20 | 0.36% | 81.0 |
| guitar@kaohsiung/fiber-wired → vocals@tainan/mobile-4g | 122.3 | 122.3 | 125.0 | 20/20 | 1.41% | 315.0 |
| vocals@tainan/mobile-4g → drums@taipei/fiber-wired | 127.7 | 127.7 | 125.2 | 18/19 | 1.23% | 273.0 |
| vocals@tainan/mobile-4g → bass@taichung/fiber-wired | 119.7 | 119.7 | 120.7 | 18/19 | 1.50% | 333.0 |
| vocals@tainan/mobile-4g → guitar@kaohsiung/fiber-wired | 127.7 | 127.7 | 125.0 | 18/19 | 1.23% | 273.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -5.86% |
| Mean worst heard RMS asynchrony | 127.2 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
