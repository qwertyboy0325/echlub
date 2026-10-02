# Jam scenario: taiwan-rock-4-mobile

> **SIMULATION.** All network, device, and musician parameters are assumed planning values, not measurements. No latency claim about real networks or people follows from this report.

- Endpoint: `native-builtin`  
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
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 29.3 | 29.3 | 31.1 | 20/20 |
| drums@taipei/fiber-wired → guitar@kaohsiung/fiber-wired | 32.0 | 32.0 | 35.3 | 20/20 |
| drums@taipei/fiber-wired → vocals@tainan/mobile-4g | 112.0 | 112.0 | 108.2 | 20/20 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 29.3 | 29.3 | 31.1 | 20/20 |
| bass@taichung/fiber-wired → guitar@kaohsiung/fiber-wired | 29.3 | 29.3 | 30.9 | 20/20 |
| bass@taichung/fiber-wired → vocals@tainan/mobile-4g | 101.3 | 101.3 | 106.4 | 20/20 |
| guitar@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 32.0 | 32.0 | 35.3 | 20/20 |
| guitar@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 29.3 | 29.3 | 30.9 | 20/20 |
| guitar@kaohsiung/fiber-wired → vocals@tainan/mobile-4g | 109.3 | 109.3 | 108.0 | 19/20 |
| vocals@tainan/mobile-4g → drums@taipei/fiber-wired | 112.0 | 112.0 | 108.2 | 18/19 |
| vocals@tainan/mobile-4g → bass@taichung/fiber-wired | 112.0 | 112.0 | 106.4 | 18/19 |
| vocals@tainan/mobile-4g → guitar@kaohsiung/fiber-wired | 112.0 | 112.0 | 108.0 | 18/19 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -4.55% |
| Mean worst heard RMS asynchrony | 111.1 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
