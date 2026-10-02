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
| 1 | bass@kaohsiung/fiber-wired/phone-ios-app | 2 | 3.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired/phone-ios-app → bass@kaohsiung/fiber-wired/phone-ios-app | 27.2 | 27.2 | 30.5 | 20/20 | 0.01% | 3.0 |
| bass@kaohsiung/fiber-wired/phone-ios-app → drums@taipei/fiber-wired/phone-ios-app | 27.2 | 27.2 | 30.5 | 20/20 | 0.00% | 0.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -1.79% |
| Mean worst heard RMS asynchrony | 35.0 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
