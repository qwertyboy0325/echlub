# Jam scenario: taiwan-rock-4-mobile

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `browser-worklet-tuned`  
- Topology: Forward  
- Cross-ISP penalty: false  
- Tempo: 120 BPM, 32 bars  
- Ensemble runs: 20

## Players

| # | Player | Jitter depth (frames) | Link expected one-way (ms) |
| --- | --- | --- | --- |
| 0 | drums@taipei/fiber-wired | 3 | 4.1 |
| 1 | bass@taichung/fiber-wired | 2 | 2.3 |
| 2 | guitar@kaohsiung/fiber-wired | 3 | 3.9 |
| 3 | vocals@tainan/mobile-4g | 19 | 34.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected |
| --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 39.7 | 39.7 | 41.4 | 20/20 |
| drums@taipei/fiber-wired → guitar@kaohsiung/fiber-wired | 42.3 | 42.3 | 45.7 | 20/20 |
| drums@taipei/fiber-wired → vocals@tainan/mobile-4g | 122.3 | 122.3 | 118.5 | 20/20 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 39.7 | 39.7 | 41.4 | 20/20 |
| bass@taichung/fiber-wired → guitar@kaohsiung/fiber-wired | 39.7 | 39.7 | 41.2 | 20/20 |
| bass@taichung/fiber-wired → vocals@tainan/mobile-4g | 111.7 | 111.7 | 116.7 | 20/20 |
| guitar@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 42.3 | 42.3 | 45.7 | 20/20 |
| guitar@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 39.7 | 39.7 | 41.2 | 20/20 |
| guitar@kaohsiung/fiber-wired → vocals@tainan/mobile-4g | 119.7 | 119.7 | 118.3 | 19/20 |
| vocals@tainan/mobile-4g → drums@taipei/fiber-wired | 122.3 | 122.3 | 118.5 | 18/19 |
| vocals@tainan/mobile-4g → bass@taichung/fiber-wired | 122.3 | 122.3 | 116.7 | 18/19 |
| vocals@tainan/mobile-4g → guitar@kaohsiung/fiber-wired | 122.3 | 122.3 | 118.3 | 18/19 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -5.52% |
| Mean worst heard RMS asynchrony | 123.4 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
