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
| 0 | drums@taipei/mobile-5g-sa/phone-ios-app | 3 | 10.8 |
| 1 | bass@taichung/mobile-5g-sa/phone-ios-app | 3 | 9.0 |
| 2 | guitar@tainan/mobile-5g-sa/phone-ios-app | 3 | 10.1 |
| 3 | vocals@kaohsiung/mobile-5g-sa/phone-ios-app | 3 | 10.6 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/mobile-5g-sa/phone-ios-app → bass@taichung/mobile-5g-sa/phone-ios-app | 43.2 | 43.2 | 47.6 | 20/20 | 0.21% | 48.0 |
| drums@taipei/mobile-5g-sa/phone-ios-app → guitar@tainan/mobile-5g-sa/phone-ios-app | 45.8 | 45.8 | 48.7 | 20/20 | 0.09% | 21.0 |
| drums@taipei/mobile-5g-sa/phone-ios-app → vocals@kaohsiung/mobile-5g-sa/phone-ios-app | 45.8 | 45.8 | 49.2 | 20/20 | 0.03% | 6.0 |
| bass@taichung/mobile-5g-sa/phone-ios-app → drums@taipei/mobile-5g-sa/phone-ios-app | 43.2 | 43.2 | 47.6 | 20/20 | 0.12% | 27.0 |
| bass@taichung/mobile-5g-sa/phone-ios-app → guitar@tainan/mobile-5g-sa/phone-ios-app | 40.5 | 40.5 | 46.9 | 20/20 | 0.41% | 93.0 |
| bass@taichung/mobile-5g-sa/phone-ios-app → vocals@kaohsiung/mobile-5g-sa/phone-ios-app | 40.5 | 40.5 | 47.4 | 20/20 | 0.63% | 141.0 |
| guitar@tainan/mobile-5g-sa/phone-ios-app → drums@taipei/mobile-5g-sa/phone-ios-app | 45.8 | 45.8 | 48.7 | 20/20 | 0.07% | 15.0 |
| guitar@tainan/mobile-5g-sa/phone-ios-app → bass@taichung/mobile-5g-sa/phone-ios-app | 43.2 | 43.2 | 46.9 | 20/20 | 0.08% | 18.0 |
| guitar@tainan/mobile-5g-sa/phone-ios-app → vocals@kaohsiung/mobile-5g-sa/phone-ios-app | 45.8 | 45.8 | 48.5 | 20/20 | 0.08% | 18.0 |
| vocals@kaohsiung/mobile-5g-sa/phone-ios-app → drums@taipei/mobile-5g-sa/phone-ios-app | 43.2 | 43.2 | 49.2 | 19/19 | 0.35% | 78.0 |
| vocals@kaohsiung/mobile-5g-sa/phone-ios-app → bass@taichung/mobile-5g-sa/phone-ios-app | 43.2 | 43.2 | 47.4 | 19/19 | 0.04% | 9.0 |
| vocals@kaohsiung/mobile-5g-sa/phone-ios-app → guitar@tainan/mobile-5g-sa/phone-ios-app | 45.8 | 45.8 | 48.5 | 19/19 | 0.01% | 3.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -3.52% |
| Mean worst heard RMS asynchrony | 60.7 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
