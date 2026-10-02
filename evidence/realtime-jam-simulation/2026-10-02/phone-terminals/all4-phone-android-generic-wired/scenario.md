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
| 0 | drums@taipei/fiber-wired/phone-android-generic | 2 | 4.1 |
| 1 | bass@taichung/fiber-wired/phone-android-generic | 2 | 2.3 |
| 2 | guitar@tainan/fiber-wired/phone-android-generic | 2 | 3.4 |
| 3 | vocals@kaohsiung/fiber-wired/phone-android-generic | 2 | 3.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired/phone-android-generic → bass@taichung/fiber-wired/phone-android-generic | 64.3 | 64.3 | 68.7 | 20/20 | 0.04% | 9.0 |
| drums@taipei/fiber-wired/phone-android-generic → guitar@tainan/fiber-wired/phone-android-generic | 67.0 | 67.0 | 69.8 | 20/20 | 0.00% | 0.0 |
| drums@taipei/fiber-wired/phone-android-generic → vocals@kaohsiung/fiber-wired/phone-android-generic | 67.0 | 67.0 | 70.3 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-android-generic → drums@taipei/fiber-wired/phone-android-generic | 67.0 | 67.0 | 68.7 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-android-generic → guitar@tainan/fiber-wired/phone-android-generic | 64.3 | 64.3 | 68.0 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-android-generic → vocals@kaohsiung/fiber-wired/phone-android-generic | 64.3 | 64.3 | 68.5 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired/phone-android-generic → drums@taipei/fiber-wired/phone-android-generic | 67.0 | 67.0 | 69.8 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired/phone-android-generic → bass@taichung/fiber-wired/phone-android-generic | 64.3 | 64.3 | 68.0 | 20/20 | 0.01% | 3.0 |
| guitar@tainan/fiber-wired/phone-android-generic → vocals@kaohsiung/fiber-wired/phone-android-generic | 67.0 | 67.0 | 69.6 | 20/20 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-android-generic → drums@taipei/fiber-wired/phone-android-generic | 67.0 | 67.0 | 70.3 | 19/19 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-android-generic → bass@taichung/fiber-wired/phone-android-generic | 64.3 | 64.3 | 68.5 | 19/19 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-android-generic → guitar@tainan/fiber-wired/phone-android-generic | 67.0 | 67.0 | 69.6 | 19/19 | 0.00% | 0.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -5.62% |
| Mean worst heard RMS asynchrony | 92.3 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
