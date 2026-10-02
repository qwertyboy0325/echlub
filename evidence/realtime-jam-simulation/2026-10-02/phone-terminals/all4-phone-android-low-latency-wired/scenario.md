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
| 0 | drums@taipei/fiber-wired/phone-android-low-latency | 2 | 4.1 |
| 1 | bass@taichung/fiber-wired/phone-android-low-latency | 2 | 2.3 |
| 2 | guitar@tainan/fiber-wired/phone-android-low-latency | 2 | 3.4 |
| 3 | vocals@kaohsiung/fiber-wired/phone-android-low-latency | 2 | 3.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired/phone-android-low-latency → bass@taichung/fiber-wired/phone-android-low-latency | 28.8 | 28.8 | 33.2 | 20/20 | 0.04% | 9.0 |
| drums@taipei/fiber-wired/phone-android-low-latency → guitar@tainan/fiber-wired/phone-android-low-latency | 31.5 | 31.5 | 34.3 | 20/20 | 0.00% | 0.0 |
| drums@taipei/fiber-wired/phone-android-low-latency → vocals@kaohsiung/fiber-wired/phone-android-low-latency | 31.5 | 31.5 | 34.8 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-android-low-latency → drums@taipei/fiber-wired/phone-android-low-latency | 31.5 | 31.5 | 33.2 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-android-low-latency → guitar@tainan/fiber-wired/phone-android-low-latency | 28.8 | 28.8 | 32.5 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired/phone-android-low-latency → vocals@kaohsiung/fiber-wired/phone-android-low-latency | 28.8 | 28.8 | 33.0 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired/phone-android-low-latency → drums@taipei/fiber-wired/phone-android-low-latency | 31.5 | 31.5 | 34.3 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired/phone-android-low-latency → bass@taichung/fiber-wired/phone-android-low-latency | 28.8 | 28.8 | 32.5 | 20/20 | 0.01% | 3.0 |
| guitar@tainan/fiber-wired/phone-android-low-latency → vocals@kaohsiung/fiber-wired/phone-android-low-latency | 31.5 | 31.5 | 34.1 | 20/20 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-android-low-latency → drums@taipei/fiber-wired/phone-android-low-latency | 31.5 | 31.5 | 34.8 | 19/19 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-android-low-latency → bass@taichung/fiber-wired/phone-android-low-latency | 28.8 | 28.8 | 33.0 | 19/19 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-android-low-latency → guitar@tainan/fiber-wired/phone-android-low-latency | 31.5 | 31.5 | 34.1 | 19/19 | 0.00% | 0.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -2.20% |
| Mean worst heard RMS asynchrony | 42.1 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
