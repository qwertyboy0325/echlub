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
| 0 | drums@taichung/fiber-wired/phone-ios-app | 2 | 2.3 |
| 1 | bass@taichung/fiber-wired/phone-ios-app | 2 | 2.3 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taichung/fiber-wired/phone-ios-app → bass@taichung/fiber-wired/phone-ios-app | 24.5 | 24.5 | 27.1 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-ios-app → drums@taichung/fiber-wired/phone-ios-app | 24.5 | 24.5 | 27.1 | 20/20 | 0.00% | 0.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -1.54% |
| Mean worst heard RMS asynchrony | 31.5 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
