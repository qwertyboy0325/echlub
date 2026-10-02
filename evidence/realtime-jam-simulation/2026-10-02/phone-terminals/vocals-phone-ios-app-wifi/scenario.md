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
| 0 | drums@taipei/fiber-wired | 2 | 4.1 |
| 1 | bass@taichung/fiber-wired | 2 | 2.3 |
| 2 | guitar@tainan/fiber-wired | 2 | 3.4 |
| 3 | vocals@kaohsiung/wifi/phone-ios-app | 3 | 7.6 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 15.0 | 15.0 | 19.4 | 20/20 | 0.05% | 12.0 |
| drums@taipei/fiber-wired → guitar@tainan/fiber-wired | 17.7 | 17.7 | 20.5 | 20/20 | 0.00% | 0.0 |
| drums@taipei/fiber-wired → vocals@kaohsiung/wifi/phone-ios-app | 30.7 | 30.7 | 35.0 | 20/20 | 0.15% | 33.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 17.7 | 17.7 | 19.4 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired → guitar@tainan/fiber-wired | 15.0 | 15.0 | 18.7 | 20/20 | 0.01% | 3.0 |
| bass@taichung/fiber-wired → vocals@kaohsiung/wifi/phone-ios-app | 28.0 | 28.0 | 33.2 | 20/20 | 0.20% | 45.0 |
| guitar@tainan/fiber-wired → drums@taipei/fiber-wired | 17.7 | 17.7 | 20.5 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired → bass@taichung/fiber-wired | 15.0 | 15.0 | 18.7 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired → vocals@kaohsiung/wifi/phone-ios-app | 30.7 | 30.7 | 34.3 | 20/20 | 0.15% | 33.0 |
| vocals@kaohsiung/wifi/phone-ios-app → drums@taipei/fiber-wired | 30.2 | 30.2 | 34.5 | 19/19 | 0.11% | 24.0 |
| vocals@kaohsiung/wifi/phone-ios-app → bass@taichung/fiber-wired | 30.2 | 30.2 | 32.7 | 19/19 | 0.04% | 9.0 |
| vocals@kaohsiung/wifi/phone-ios-app → guitar@tainan/fiber-wired | 30.2 | 30.2 | 33.8 | 19/19 | 0.09% | 21.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -1.20% |
| Mean worst heard RMS asynchrony | 33.1 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
