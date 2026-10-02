# Jam scenario: taiwan-trio

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `browser-webrtc-default`  
- Topology: Forward  
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
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 110.6 | 110.6 | 112.4 | 20/20 | 0.15% | 33.0 |
| drums@taipei/fiber-wired → guitar@kaohsiung/cable-wired | 113.3 | 113.3 | 118.2 | 20/20 | 0.16% | 36.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 108.0 | 108.0 | 109.7 | 20/20 | 0.07% | 15.0 |
| bass@taichung/fiber-wired → guitar@kaohsiung/cable-wired | 110.6 | 110.6 | 113.7 | 20/20 | 0.15% | 33.0 |
| guitar@kaohsiung/cable-wired → drums@taipei/fiber-wired | 121.3 | 121.3 | 123.5 | 20/20 | 0.24% | 54.0 |
| guitar@kaohsiung/cable-wired → bass@taichung/fiber-wired | 121.3 | 121.3 | 121.7 | 20/20 | 0.15% | 33.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -9.77% |
| Mean worst heard RMS asynchrony | 163.2 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
