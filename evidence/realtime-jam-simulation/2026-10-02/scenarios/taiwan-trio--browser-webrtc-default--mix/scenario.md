# Jam scenario: taiwan-trio

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
| 2 | guitar@kaohsiung/cable-wired | 5 | 8.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 110.6 | 110.6 | 119.0 | 20/20 | 0.40% | 90.0 |
| drums@taipei/fiber-wired → guitar@kaohsiung/cable-wired | 118.6 | 118.6 | 124.8 | 20/20 | 0.19% | 42.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 113.3 | 113.3 | 116.4 | 20/20 | 0.08% | 18.0 |
| bass@taichung/fiber-wired → guitar@kaohsiung/cable-wired | 116.0 | 116.0 | 120.4 | 20/20 | 0.13% | 30.0 |
| guitar@kaohsiung/cable-wired → drums@taipei/fiber-wired | 126.6 | 126.6 | 130.2 | 20/20 | 0.20% | 45.0 |
| guitar@kaohsiung/cable-wired → bass@taichung/fiber-wired | 121.3 | 121.3 | 128.4 | 20/20 | 0.47% | 105.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -10.07% |
| Mean worst heard RMS asynchrony | 168.8 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
