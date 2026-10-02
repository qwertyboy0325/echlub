# Jam scenario: taiwan-rock-4-wired

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
| 2 | guitar@tainan/fiber-wired | 3 | 3.4 |
| 3 | vocals@kaohsiung/fiber-wired | 3 | 3.9 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected |
| --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 110.6 | 110.6 | 112.4 | 20/20 |
| drums@taipei/fiber-wired → guitar@tainan/fiber-wired | 110.6 | 110.6 | 113.5 | 20/20 |
| drums@taipei/fiber-wired → vocals@kaohsiung/fiber-wired | 110.6 | 110.6 | 114.0 | 20/20 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 108.0 | 108.0 | 109.7 | 20/20 |
| bass@taichung/fiber-wired → guitar@tainan/fiber-wired | 108.0 | 108.0 | 109.0 | 20/20 |
| bass@taichung/fiber-wired → vocals@kaohsiung/fiber-wired | 108.0 | 108.0 | 109.5 | 20/20 |
| guitar@tainan/fiber-wired → drums@taipei/fiber-wired | 110.6 | 110.6 | 113.5 | 20/20 |
| guitar@tainan/fiber-wired → bass@taichung/fiber-wired | 110.6 | 110.6 | 111.7 | 20/20 |
| guitar@tainan/fiber-wired → vocals@kaohsiung/fiber-wired | 110.6 | 110.6 | 113.3 | 20/20 |
| vocals@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 110.6 | 110.6 | 114.0 | 19/19 |
| vocals@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 108.0 | 108.0 | 112.2 | 19/19 |
| vocals@kaohsiung/fiber-wired → guitar@tainan/fiber-wired | 110.6 | 110.6 | 113.3 | 19/19 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -9.54% |
| Mean worst heard RMS asynchrony | 156.0 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
