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
| 0 | drums@taipei/mobile-5g/phone-ios-app | 4 | 17.8 |
| 1 | bass@taichung/mobile-5g/phone-ios-app | 4 | 16.0 |
| 2 | guitar@tainan/mobile-5g/phone-ios-app | 4 | 17.1 |
| 3 | vocals@kaohsiung/mobile-5g/phone-ios-app | 4 | 17.6 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/mobile-5g/phone-ios-app → bass@taichung/mobile-5g/phone-ios-app | 61.8 | 61.8 | 69.6 | 20/20 | 0.71% | 156.0 |
| drums@taipei/mobile-5g/phone-ios-app → guitar@tainan/mobile-5g/phone-ios-app | 64.5 | 64.5 | 70.7 | 20/20 | 0.59% | 132.0 |
| drums@taipei/mobile-5g/phone-ios-app → vocals@kaohsiung/mobile-5g/phone-ios-app | 69.8 | 69.8 | 71.2 | 20/20 | 0.13% | 30.0 |
| bass@taichung/mobile-5g/phone-ios-app → drums@taipei/mobile-5g/phone-ios-app | 61.8 | 61.8 | 69.6 | 20/20 | 0.68% | 153.0 |
| bass@taichung/mobile-5g/phone-ios-app → guitar@tainan/mobile-5g/phone-ios-app | 59.2 | 59.2 | 68.9 | 20/20 | 1.16% | 261.0 |
| bass@taichung/mobile-5g/phone-ios-app → vocals@kaohsiung/mobile-5g/phone-ios-app | 64.5 | 64.5 | 69.4 | 20/20 | 0.33% | 75.0 |
| guitar@tainan/mobile-5g/phone-ios-app → drums@taipei/mobile-5g/phone-ios-app | 61.8 | 61.8 | 70.7 | 20/20 | 0.96% | 207.0 |
| guitar@tainan/mobile-5g/phone-ios-app → bass@taichung/mobile-5g/phone-ios-app | 61.8 | 61.8 | 68.9 | 20/20 | 0.52% | 117.0 |
| guitar@tainan/mobile-5g/phone-ios-app → vocals@kaohsiung/mobile-5g/phone-ios-app | 61.8 | 61.8 | 70.5 | 19/20 | 0.90% | 198.0 |
| vocals@kaohsiung/mobile-5g/phone-ios-app → drums@taipei/mobile-5g/phone-ios-app | 69.8 | 69.8 | 71.2 | 19/19 | 0.09% | 21.0 |
| vocals@kaohsiung/mobile-5g/phone-ios-app → bass@taichung/mobile-5g/phone-ios-app | 64.5 | 64.5 | 69.4 | 19/19 | 0.20% | 45.0 |
| vocals@kaohsiung/mobile-5g/phone-ios-app → guitar@tainan/mobile-5g/phone-ios-app | 64.5 | 64.5 | 70.5 | 19/19 | 0.41% | 93.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -5.39% |
| Mean worst heard RMS asynchrony | 89.8 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
