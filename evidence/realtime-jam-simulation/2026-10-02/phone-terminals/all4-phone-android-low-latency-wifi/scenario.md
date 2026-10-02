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
| 0 | drums@taipei/wifi/phone-android-low-latency | 3 | 7.8 |
| 1 | bass@taichung/wifi/phone-android-low-latency | 3 | 6.0 |
| 2 | guitar@tainan/wifi/phone-android-low-latency | 3 | 7.1 |
| 3 | vocals@kaohsiung/wifi/phone-android-low-latency | 3 | 7.6 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/wifi/phone-android-low-latency → bass@taichung/wifi/phone-android-low-latency | 44.8 | 44.8 | 48.6 | 20/20 | 0.17% | 39.0 |
| drums@taipei/wifi/phone-android-low-latency → guitar@tainan/wifi/phone-android-low-latency | 42.2 | 42.2 | 49.7 | 20/20 | 0.87% | 192.0 |
| drums@taipei/wifi/phone-android-low-latency → vocals@kaohsiung/wifi/phone-android-low-latency | 44.8 | 44.8 | 50.2 | 20/20 | 0.37% | 84.0 |
| bass@taichung/wifi/phone-android-low-latency → drums@taipei/wifi/phone-android-low-latency | 44.8 | 44.8 | 48.6 | 20/20 | 0.24% | 54.0 |
| bass@taichung/wifi/phone-android-low-latency → guitar@tainan/wifi/phone-android-low-latency | 39.5 | 39.5 | 47.9 | 20/20 | 1.23% | 276.0 |
| bass@taichung/wifi/phone-android-low-latency → vocals@kaohsiung/wifi/phone-android-low-latency | 42.2 | 42.2 | 48.4 | 20/20 | 0.48% | 108.0 |
| guitar@tainan/wifi/phone-android-low-latency → drums@taipei/wifi/phone-android-low-latency | 44.8 | 44.8 | 49.7 | 20/20 | 0.31% | 66.0 |
| guitar@tainan/wifi/phone-android-low-latency → bass@taichung/wifi/phone-android-low-latency | 39.5 | 39.5 | 47.9 | 20/20 | 1.12% | 246.0 |
| guitar@tainan/wifi/phone-android-low-latency → vocals@kaohsiung/wifi/phone-android-low-latency | 47.5 | 47.5 | 49.5 | 19/20 | 0.13% | 30.0 |
| vocals@kaohsiung/wifi/phone-android-low-latency → drums@taipei/wifi/phone-android-low-latency | 47.5 | 47.5 | 50.2 | 19/19 | 0.09% | 21.0 |
| vocals@kaohsiung/wifi/phone-android-low-latency → bass@taichung/wifi/phone-android-low-latency | 44.8 | 44.8 | 48.4 | 19/19 | 0.11% | 24.0 |
| vocals@kaohsiung/wifi/phone-android-low-latency → guitar@tainan/wifi/phone-android-low-latency | 47.5 | 47.5 | 49.5 | 19/19 | 0.07% | 15.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -3.53% |
| Mean worst heard RMS asynchrony | 61.2 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
