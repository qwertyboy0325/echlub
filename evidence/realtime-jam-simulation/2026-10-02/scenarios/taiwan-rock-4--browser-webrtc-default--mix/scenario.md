# Jam scenario: taiwan-rock-4

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `browser-webrtc-default`  
- Topology: Mix  
- Cross-ISP penalty: false  
- Tempo: 120 BPM, 32 bars  
- Ensemble runs: 20

## Players

| # | Player | Jitter depth (frames) | Link expected one-way (ms) |
| --- | --- | --- | --- |
| 0 | drums@taipei/fiber-wired | 3 | 4.1 |
| 1 | bass@taichung/fiber-wired | 2 | 2.3 |
| 2 | guitar@tainan/cable-wired | 5 | 7.6 |
| 3 | vocals@hualien/wifi | 8 | 10.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 110.6 | 110.6 | 119.0 | 20/20 | 0.40% | 90.0 |
| drums@taipei/fiber-wired → guitar@tainan/cable-wired | 118.6 | 118.6 | 124.3 | 20/20 | 0.17% | 39.0 |
| drums@taipei/fiber-wired → vocals@hualien/wifi | 121.3 | 121.3 | 126.8 | 20/20 | 0.63% | 141.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 113.3 | 113.3 | 116.4 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@tainan/cable-wired | 116.0 | 116.0 | 119.9 | 20/20 | 0.12% | 27.0 |
| bass@taichung/fiber-wired → vocals@hualien/wifi | 118.6 | 118.6 | 122.4 | 20/20 | 0.57% | 129.0 |
| guitar@tainan/cable-wired → drums@taipei/fiber-wired | 126.6 | 126.6 | 129.7 | 20/20 | 0.19% | 42.0 |
| guitar@tainan/cable-wired → bass@taichung/fiber-wired | 121.3 | 121.3 | 127.9 | 20/20 | 0.45% | 102.0 |
| guitar@tainan/cable-wired → vocals@hualien/wifi | 132.0 | 132.0 | 135.7 | 19/20 | 0.68% | 153.0 |
| vocals@hualien/wifi → drums@taipei/fiber-wired | 137.3 | 137.3 | 140.2 | 19/19 | 0.73% | 165.0 |
| vocals@hualien/wifi → bass@taichung/fiber-wired | 132.0 | 132.0 | 138.4 | 19/19 | 1.00% | 225.0 |
| vocals@hualien/wifi → guitar@tainan/cable-wired | 140.0 | 140.0 | 143.7 | 19/19 | 0.77% | 174.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -10.48% |
| Mean worst heard RMS asynchrony | 175.2 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
