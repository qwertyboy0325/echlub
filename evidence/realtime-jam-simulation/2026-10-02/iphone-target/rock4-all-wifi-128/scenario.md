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
| 0 | drums@taipei/wifi/phone-ios-app-128 | 3 | 7.8 |
| 1 | bass@taichung/wifi/phone-ios-app-128 | 3 | 6.0 |
| 2 | guitar@tainan/wifi/phone-ios-app-128 | 3 | 7.1 |
| 3 | vocals@kaohsiung/wifi/phone-ios-app-128 | 3 | 7.6 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/wifi/phone-ios-app-128 → bass@taichung/wifi/phone-ios-app-128 | 35.2 | 35.2 | 39.0 | 20/20 | 0.17% | 39.0 |
| drums@taipei/wifi/phone-ios-app-128 → guitar@tainan/wifi/phone-ios-app-128 | 32.5 | 32.5 | 40.1 | 20/20 | 0.87% | 192.0 |
| drums@taipei/wifi/phone-ios-app-128 → vocals@kaohsiung/wifi/phone-ios-app-128 | 35.2 | 35.2 | 40.6 | 20/20 | 0.37% | 84.0 |
| bass@taichung/wifi/phone-ios-app-128 → drums@taipei/wifi/phone-ios-app-128 | 35.2 | 35.2 | 39.0 | 20/20 | 0.24% | 54.0 |
| bass@taichung/wifi/phone-ios-app-128 → guitar@tainan/wifi/phone-ios-app-128 | 29.8 | 29.8 | 38.3 | 20/20 | 1.23% | 276.0 |
| bass@taichung/wifi/phone-ios-app-128 → vocals@kaohsiung/wifi/phone-ios-app-128 | 32.5 | 32.5 | 38.8 | 20/20 | 0.48% | 108.0 |
| guitar@tainan/wifi/phone-ios-app-128 → drums@taipei/wifi/phone-ios-app-128 | 35.2 | 35.2 | 40.1 | 20/20 | 0.31% | 66.0 |
| guitar@tainan/wifi/phone-ios-app-128 → bass@taichung/wifi/phone-ios-app-128 | 29.8 | 29.8 | 38.3 | 20/20 | 1.12% | 246.0 |
| guitar@tainan/wifi/phone-ios-app-128 → vocals@kaohsiung/wifi/phone-ios-app-128 | 37.8 | 37.8 | 39.9 | 19/20 | 0.13% | 30.0 |
| vocals@kaohsiung/wifi/phone-ios-app-128 → drums@taipei/wifi/phone-ios-app-128 | 37.8 | 37.8 | 40.6 | 19/19 | 0.09% | 21.0 |
| vocals@kaohsiung/wifi/phone-ios-app-128 → bass@taichung/wifi/phone-ios-app-128 | 35.2 | 35.2 | 38.8 | 19/19 | 0.11% | 24.0 |
| vocals@kaohsiung/wifi/phone-ios-app-128 → guitar@tainan/wifi/phone-ios-app-128 | 37.8 | 37.8 | 39.9 | 19/19 | 0.07% | 15.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -2.58% |
| Mean worst heard RMS asynchrony | 47.7 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
