# Jam scenario: custom

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `native-interface`  
- Topology: Forward  
- Cross-ISP penalty: false  
- Tempo: 120 BPM, 32 bars  
- Ensemble runs: 20

## Players

| # | Player | Jitter depth (frames) | Link expected one-way (ms) |
| --- | --- | --- | --- |
| 0 | drums@taipei/mobile-4g/phone-ios-app | 7 | 34.8 |
| 1 | bass@taichung/mobile-4g/phone-ios-app | 7 | 33.0 |
| 2 | guitar@tainan/mobile-4g/phone-ios-app | 7 | 34.1 |
| 3 | vocals@kaohsiung/mobile-4g/phone-ios-app | 7 | 34.6 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/mobile-4g/phone-ios-app → bass@taichung/mobile-4g/phone-ios-app | 112.5 | 112.5 | 119.6 | 20/20 | 0.38% | 84.0 |
| drums@taipei/mobile-4g/phone-ios-app → guitar@tainan/mobile-4g/phone-ios-app | 112.5 | 112.5 | 120.7 | 20/20 | 0.39% | 87.0 |
| drums@taipei/mobile-4g/phone-ios-app → vocals@kaohsiung/mobile-4g/phone-ios-app | 107.2 | 107.2 | 121.2 | 20/20 | 1.07% | 237.0 |
| bass@taichung/mobile-4g/phone-ios-app → drums@taipei/mobile-4g/phone-ios-app | 109.8 | 109.8 | 119.6 | 20/20 | 0.54% | 117.0 |
| bass@taichung/mobile-4g/phone-ios-app → guitar@tainan/mobile-4g/phone-ios-app | 109.8 | 109.8 | 118.9 | 19/20 | 0.52% | 117.0 |
| bass@taichung/mobile-4g/phone-ios-app → vocals@kaohsiung/mobile-4g/phone-ios-app | 112.5 | 112.5 | 119.4 | 20/20 | 0.50% | 111.0 |
| guitar@tainan/mobile-4g/phone-ios-app → drums@taipei/mobile-4g/phone-ios-app | 107.2 | 107.2 | 120.7 | 20/20 | 1.11% | 246.0 |
| guitar@tainan/mobile-4g/phone-ios-app → bass@taichung/mobile-4g/phone-ios-app | 109.8 | 109.8 | 118.9 | 20/20 | 0.63% | 138.0 |
| guitar@tainan/mobile-4g/phone-ios-app → vocals@kaohsiung/mobile-4g/phone-ios-app | 112.5 | 112.5 | 120.5 | 20/20 | 0.47% | 105.0 |
| vocals@kaohsiung/mobile-4g/phone-ios-app → drums@taipei/mobile-4g/phone-ios-app | 109.8 | 109.8 | 121.2 | 19/19 | 0.72% | 159.0 |
| vocals@kaohsiung/mobile-4g/phone-ios-app → bass@taichung/mobile-4g/phone-ios-app | 109.8 | 109.8 | 119.4 | 19/19 | 0.58% | 129.0 |
| vocals@kaohsiung/mobile-4g/phone-ios-app → guitar@tainan/mobile-4g/phone-ios-app | 112.5 | 112.5 | 120.5 | 19/19 | 0.46% | 102.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -9.59% |
| Mean worst heard RMS asynchrony | 156.5 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
