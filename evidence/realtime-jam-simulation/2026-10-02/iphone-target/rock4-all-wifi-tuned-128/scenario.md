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
| 0 | drums@taipei/wifi-tuned/phone-ios-app-128 | 2 | 5.1 |
| 1 | bass@taichung/wifi-tuned/phone-ios-app-128 | 2 | 3.3 |
| 2 | guitar@tainan/wifi-tuned/phone-ios-app-128 | 2 | 4.4 |
| 3 | vocals@kaohsiung/wifi-tuned/phone-ios-app-128 | 2 | 4.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/wifi-tuned/phone-ios-app-128 → bass@taichung/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 28.2 | 20/20 | 0.01% | 3.0 |
| drums@taipei/wifi-tuned/phone-ios-app-128 → guitar@tainan/wifi-tuned/phone-ios-app-128 | 27.2 | 27.2 | 29.3 | 20/20 | 0.00% | 0.0 |
| drums@taipei/wifi-tuned/phone-ios-app-128 → vocals@kaohsiung/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 29.8 | 20/20 | 0.04% | 9.0 |
| bass@taichung/wifi-tuned/phone-ios-app-128 → drums@taipei/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 28.2 | 20/20 | 0.01% | 3.0 |
| bass@taichung/wifi-tuned/phone-ios-app-128 → guitar@tainan/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 27.5 | 20/20 | 0.00% | 0.0 |
| bass@taichung/wifi-tuned/phone-ios-app-128 → vocals@kaohsiung/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 28.0 | 20/20 | 0.01% | 3.0 |
| guitar@tainan/wifi-tuned/phone-ios-app-128 → drums@taipei/wifi-tuned/phone-ios-app-128 | 27.2 | 27.2 | 29.3 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/wifi-tuned/phone-ios-app-128 → bass@taichung/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 27.5 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/wifi-tuned/phone-ios-app-128 → vocals@kaohsiung/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 29.1 | 20/20 | 0.08% | 18.0 |
| vocals@kaohsiung/wifi-tuned/phone-ios-app-128 → drums@taipei/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 29.8 | 19/19 | 0.09% | 21.0 |
| vocals@kaohsiung/wifi-tuned/phone-ios-app-128 → bass@taichung/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 28.0 | 19/19 | 0.01% | 3.0 |
| vocals@kaohsiung/wifi-tuned/phone-ios-app-128 → guitar@tainan/wifi-tuned/phone-ios-app-128 | 24.5 | 24.5 | 29.1 | 19/19 | 0.01% | 3.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -1.66% |
| Mean worst heard RMS asynchrony | 35.3 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
