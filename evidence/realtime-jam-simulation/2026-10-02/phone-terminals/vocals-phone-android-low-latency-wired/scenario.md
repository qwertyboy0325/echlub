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
| 3 | vocals@kaohsiung/fiber-wired/phone-android-low-latency | 2 | 3.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 95.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 15.0 | 15.0 | 19.4 | 20/20 | 0.04% | 9.0 |
| drums@taipei/fiber-wired → guitar@tainan/fiber-wired | 17.7 | 17.7 | 20.5 | 20/20 | 0.00% | 0.0 |
| drums@taipei/fiber-wired → vocals@kaohsiung/fiber-wired/phone-android-low-latency | 27.3 | 27.3 | 30.7 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 17.7 | 17.7 | 19.4 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired → guitar@tainan/fiber-wired | 15.0 | 15.0 | 18.7 | 20/20 | 0.00% | 0.0 |
| bass@taichung/fiber-wired → vocals@kaohsiung/fiber-wired/phone-android-low-latency | 24.7 | 24.7 | 28.9 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired → drums@taipei/fiber-wired | 17.7 | 17.7 | 20.5 | 20/20 | 0.00% | 0.0 |
| guitar@tainan/fiber-wired → bass@taichung/fiber-wired | 15.0 | 15.0 | 18.7 | 20/20 | 0.01% | 3.0 |
| guitar@tainan/fiber-wired → vocals@kaohsiung/fiber-wired/phone-android-low-latency | 27.3 | 27.3 | 30.0 | 20/20 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-android-low-latency → drums@taipei/fiber-wired | 21.8 | 21.8 | 25.2 | 19/19 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-android-low-latency → bass@taichung/fiber-wired | 19.2 | 19.2 | 23.4 | 19/19 | 0.00% | 0.0 |
| vocals@kaohsiung/fiber-wired/phone-android-low-latency → guitar@tainan/fiber-wired | 21.8 | 21.8 | 24.5 | 19/19 | 0.00% | 0.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 80% |
| Playable-or-better fraction | 100% |
| Mean tempo drift | -1.01% |
| Mean worst heard RMS asynchrony | 29.4 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
