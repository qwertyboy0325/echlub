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
| 3 | vocals@kaohsiung/fiber-wired/phone-ios-app | 2 | 3.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired/phone-ios-app → bass@taichung/fiber-wired/phone-ios-app | 24.5 | 24.5 | 28.9 | 20/20 | 0.04% | 9.0 |
| drums@taipei/fiber-wired/phone-ios-app → guitar@tainan/fiber-wired/phone-ios-app | 27.2 | 27.2 | 30.0 | 20/20 | 0.00% | 0.0 |
| drums@taipei/fiber-wired/phone-ios-app → vocals@kaohsiung/fiber-wired/phone-ios-app | 27.2 | 27.2 | 30.5 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-ios-app → drums@taipei/fiber-wired/phone-ios-app | 27.2 | 27.2 | 28.9 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-ios-app → guitar@tainan/fiber-wired/phone-ios-app | 24.5 | 24.5 | 28.2 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-ios-app → vocals@kaohsiung/fiber-wired/phone-ios-app | 24.5 | 24.5 | 28.7 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired/phone-ios-app → drums@taipei/fiber-wired/phone-ios-app | 27.2 | 27.2 | 30.0 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired/phone-ios-app → bass@taichung/fiber-wired/phone-ios-app | 24.5 | 24.5 | 28.2 | 20/20 | 0.01% | 3.0 |
| guitar@tainan/fiber-wired/phone-ios-app → vocals@kaohsiung/fiber-wired/phone-ios-app | 27.2 | 27.2 | 29.8 | 20/20 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-ios-app → drums@taipei/fiber-wired/phone-ios-app | 27.2 | 27.2 | 30.5 | 19/19 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-ios-app → bass@taichung/fiber-wired/phone-ios-app | 24.5 | 24.5 | 28.7 | 19/19 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-ios-app → guitar@tainan/fiber-wired/phone-ios-app | 27.2 | 27.2 | 29.8 | 19/19 | 0.00% | 0.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -1.76% |
| Mean worst heard RMS asynchrony | 36.5 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
