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
| 0 | drums@taipei/wifi/phone-ios-app | 3 | 7.8 |
| 1 | bass@kaohsiung/wifi/phone-ios-app | 3 | 7.6 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/wifi/phone-ios-app → bass@kaohsiung/wifi/phone-ios-app | 37.8 | 37.8 | 45.9 | 20/20 | 1.03% | 225.0 |
| bass@kaohsiung/wifi/phone-ios-app → drums@taipei/wifi/phone-ios-app | 37.8 | 37.8 | 45.9 | 20/20 | 1.01% | 225.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -2.77% |
| Mean worst heard RMS asynchrony | 49.0 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
