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
| 0 | drums@taipei/fiber-wired/phone-ios-app | 2 | 4.1 |
| 1 | bass@taichung/fiber-wired/phone-ios-app | 2 | 2.3 |
| 2 | guitar@tainan/fiber-wired/phone-ios-app | 2 | 3.4 |
| 3 | vocals@kaohsiung/wifi/phone-ios-app | 3 | 7.6 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired/phone-ios-app → bass@taichung/fiber-wired/phone-ios-app | 24.5 | 24.5 | 28.9 | 20/20 | 0.05% | 12.0 |
| drums@taipei/fiber-wired/phone-ios-app → guitar@tainan/fiber-wired/phone-ios-app | 27.2 | 27.2 | 30.0 | 20/20 | 0.00% | 0.0 |
| drums@taipei/fiber-wired/phone-ios-app → vocals@kaohsiung/wifi/phone-ios-app | 35.2 | 35.2 | 39.5 | 20/20 | 0.15% | 33.0 |
| bass@taichung/fiber-wired/phone-ios-app → drums@taipei/fiber-wired/phone-ios-app | 27.2 | 27.2 | 28.9 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-ios-app → guitar@tainan/fiber-wired/phone-ios-app | 24.5 | 24.5 | 28.2 | 20/20 | 0.01% | 3.0 |
| bass@taichung/fiber-wired/phone-ios-app → vocals@kaohsiung/wifi/phone-ios-app | 32.5 | 32.5 | 37.7 | 20/20 | 0.20% | 45.0 |
| guitar@tainan/fiber-wired/phone-ios-app → drums@taipei/fiber-wired/phone-ios-app | 27.2 | 27.2 | 30.0 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired/phone-ios-app → bass@taichung/fiber-wired/phone-ios-app | 24.5 | 24.5 | 28.2 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired/phone-ios-app → vocals@kaohsiung/wifi/phone-ios-app | 35.2 | 35.2 | 38.8 | 20/20 | 0.15% | 33.0 |
| vocals@kaohsiung/wifi/phone-ios-app → drums@taipei/fiber-wired/phone-ios-app | 35.2 | 35.2 | 39.5 | 19/19 | 0.11% | 24.0 |
| vocals@kaohsiung/wifi/phone-ios-app → bass@taichung/fiber-wired/phone-ios-app | 35.2 | 35.2 | 37.7 | 19/19 | 0.04% | 9.0 |
| vocals@kaohsiung/wifi/phone-ios-app → guitar@tainan/fiber-wired/phone-ios-app | 35.2 | 35.2 | 38.8 | 19/19 | 0.09% | 21.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -2.02% |
| Mean worst heard RMS asynchrony | 41.6 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
