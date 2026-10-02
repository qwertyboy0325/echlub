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
| 0 | drums@taipei/wifi-tuned/phone-ios-app | 2 | 5.1 |
| 1 | bass@taichung/wifi-tuned/phone-ios-app | 2 | 3.3 |
| 2 | guitar@tainan/wifi-tuned/phone-ios-app | 2 | 4.4 |
| 3 | vocals@kaohsiung/wifi-tuned/phone-ios-app | 2 | 4.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/wifi-tuned/phone-ios-app → bass@taichung/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 33.6 | 20/20 | 0.01% | 3.0 |
| drums@taipei/wifi-tuned/phone-ios-app → guitar@tainan/wifi-tuned/phone-ios-app | 32.5 | 32.5 | 34.7 | 20/20 | 0.00% | 0.0 |
| drums@taipei/wifi-tuned/phone-ios-app → vocals@kaohsiung/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 35.2 | 20/20 | 0.04% | 9.0 |
| bass@taichung/wifi-tuned/phone-ios-app → drums@taipei/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 33.6 | 20/20 | 0.01% | 3.0 |
| bass@taichung/wifi-tuned/phone-ios-app → guitar@tainan/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 32.9 | 20/20 | 0.00% | 0.0 |
| bass@taichung/wifi-tuned/phone-ios-app → vocals@kaohsiung/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 33.4 | 20/20 | 0.01% | 3.0 |
| guitar@tainan/wifi-tuned/phone-ios-app → drums@taipei/wifi-tuned/phone-ios-app | 32.5 | 32.5 | 34.7 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/wifi-tuned/phone-ios-app → bass@taichung/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 32.9 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/wifi-tuned/phone-ios-app → vocals@kaohsiung/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 34.5 | 20/20 | 0.08% | 18.0 |
| vocals@kaohsiung/wifi-tuned/phone-ios-app → drums@taipei/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 35.2 | 19/19 | 0.09% | 21.0 |
| vocals@kaohsiung/wifi-tuned/phone-ios-app → bass@taichung/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 33.4 | 19/19 | 0.01% | 3.0 |
| vocals@kaohsiung/wifi-tuned/phone-ios-app → guitar@tainan/wifi-tuned/phone-ios-app | 29.8 | 29.8 | 34.5 | 19/19 | 0.01% | 3.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -2.20% |
| Mean worst heard RMS asynchrony | 42.0 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
