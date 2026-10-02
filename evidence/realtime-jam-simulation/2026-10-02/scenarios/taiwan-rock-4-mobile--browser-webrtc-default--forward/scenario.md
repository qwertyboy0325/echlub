# Jam scenario: taiwan-rock-4-mobile

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
| 2 | guitar@kaohsiung/fiber-wired | 3 | 3.9 |
| 3 | vocals@tainan/mobile-4g | 19 | 34.1 |

## Mouth-to-ear latency (pipeline simulation + analytic device budget)

Jitter-buffer coverage: 99.0%

| From → To | p50 (ms) | p95 (ms) | Analytic budget (ms) | Probes detected | Concealed | Dropouts/min |
| --- | --- | --- | --- | --- | --- | --- |
| drums@taipei/fiber-wired → bass@taichung/fiber-wired | 110.6 | 110.6 | 112.4 | 20/20 | 0.13% | 30.0 |
| drums@taipei/fiber-wired → guitar@kaohsiung/fiber-wired | 110.6 | 110.6 | 114.0 | 20/20 | 0.13% | 30.0 |
| drums@taipei/fiber-wired → vocals@tainan/mobile-4g | 158.6 | 158.6 | 154.8 | 20/20 | 1.17% | 255.0 |
| bass@taichung/fiber-wired → drums@taipei/fiber-wired | 108.0 | 108.0 | 109.7 | 20/20 | 0.04% | 9.0 |
| bass@taichung/fiber-wired → guitar@kaohsiung/fiber-wired | 108.0 | 108.0 | 109.5 | 20/20 | 0.07% | 15.0 |
| bass@taichung/fiber-wired → vocals@tainan/mobile-4g | 148.0 | 148.0 | 153.0 | 20/20 | 1.22% | 270.0 |
| guitar@kaohsiung/fiber-wired → drums@taipei/fiber-wired | 110.6 | 110.6 | 114.0 | 20/20 | 0.12% | 27.0 |
| guitar@kaohsiung/fiber-wired → bass@taichung/fiber-wired | 110.6 | 110.6 | 112.2 | 20/20 | 0.09% | 21.0 |
| guitar@kaohsiung/fiber-wired → vocals@tainan/mobile-4g | 156.0 | 156.0 | 154.6 | 19/20 | 1.23% | 276.0 |
| vocals@tainan/mobile-4g → drums@taipei/fiber-wired | 190.6 | 190.6 | 186.8 | 18/19 | 1.23% | 273.0 |
| vocals@tainan/mobile-4g → bass@taichung/fiber-wired | 193.3 | 193.3 | 187.7 | 18/19 | 1.17% | 258.0 |
| vocals@tainan/mobile-4g → guitar@kaohsiung/fiber-wired | 190.6 | 190.6 | 186.6 | 18/19 | 1.17% | 258.0 |

## Simulated ensemble

| Metric | Value |
| --- | --- |
| Tight fraction | 0% |
| Playable-or-better fraction | 0% |
| Mean tempo drift | -11.16% |
| Mean worst heard RMS asynchrony | 201.7 ms |

Verdict thresholds (hypotheses): Tight = |drift| ≤ 2% and heard RMS ≤ 30 ms; Playable = |drift| ≤ 5% and heard RMS ≤ 45 ms.
