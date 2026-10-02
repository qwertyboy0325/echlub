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
| 0 | drums@taipei/wifi/phone-android-generic | 3 | 7.8 |
| 1 | bass@taichung/wifi/phone-android-generic | 3 | 6.0 |
| 2 | guitar@tainan/wifi/phone-android-generic | 3 | 7.1 |
| 3 | vocals@kaohsiung/wifi/phone-android-generic | 3 | 7.6 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/wifi/phone-android-generic → bass@taichung/wifi/phone-android-generic | 80.3 | 80.3 | 84.1 | 20/20 | 0.17% | 39.0 |
| drums@taipei/wifi/phone-android-generic → guitar@tainan/wifi/phone-android-generic | 77.7 | 77.7 | 85.2 | 20/20 | 0.87% | 192.0 |
| drums@taipei/wifi/phone-android-generic → vocals@kaohsiung/wifi/phone-android-generic | 80.3 | 80.3 | 85.7 | 20/20 | 0.37% | 84.0 |
| bass@taichung/wifi/phone-android-generic → drums@taipei/wifi/phone-android-generic | 80.3 | 80.3 | 84.1 | 20/20 | 0.24% | 54.0 |
| bass@taichung/wifi/phone-android-generic → guitar@tainan/wifi/phone-android-generic | 75.0 | 75.0 | 83.4 | 20/20 | 1.23% | 276.0 |
| bass@taichung/wifi/phone-android-generic → vocals@kaohsiung/wifi/phone-android-generic | 77.7 | 77.7 | 83.9 | 20/20 | 0.48% | 108.0 |
| guitar@tainan/wifi/phone-android-generic → drums@taipei/wifi/phone-android-generic | 80.3 | 80.3 | 85.2 | 20/20 | 0.31% | 66.0 |
| guitar@tainan/wifi/phone-android-generic → bass@taichung/wifi/phone-android-generic | 75.0 | 75.0 | 83.4 | 20/20 | 1.12% | 246.0 |
| guitar@tainan/wifi/phone-android-generic → vocals@kaohsiung/wifi/phone-android-generic | 83.0 | 83.0 | 85.0 | 19/20 | 0.13% | 30.0 |
| vocals@kaohsiung/wifi/phone-android-generic → drums@taipei/wifi/phone-android-generic | 83.0 | 83.0 | 85.7 | 19/19 | 0.09% | 21.0 |
| vocals@kaohsiung/wifi/phone-android-generic → bass@taichung/wifi/phone-android-generic | 80.3 | 80.3 | 83.9 | 19/19 | 0.11% | 24.0 |
| vocals@kaohsiung/wifi/phone-android-generic → guitar@tainan/wifi/phone-android-generic | 83.0 | 83.0 | 85.0 | 19/19 | 0.07% | 15.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -6.87% |
| Mean worst heard RMS asynchrony | 112.1 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
