# Next Work Packages

Direction: real-time jamming across Taiwan (ADR-0006). Each package has a
stop gate; simulations never substitute for measurement.

## ECHLUB-REALTIME-JAM-PROTOTYPE-01

- Jam core, relay prototype, simulated musicians, UDP bots
- **Status**: implemented (simulation + localhost only)

## ECHLUB-NATIVE-AUDIO-CLIENT-01 (next)

- Native client on the jam core: audio device I/O (64–128 sample buffers),
  device-callback clock, local monitoring, `forward` and `mix` relay modes
- Hardware loopback measurement of real mouth-to-ear latency
- First two-player human session on LAN, then across cities

## ECHLUB-TAIWAN-NETWORK-BASELINE-01

- Relay in central Taiwan; probe agents at 3–5 sites (Taipei, Taichung,
  Tainan/Kaohsiung, Hualien) across ISPs and access types
- Replace `Assumed` profiles in `echlub-jam` with measured ones
- Gate: p95 one-way to relay on wired fibre

## ECHLUB-BROWSER-JAM-PATH-01

- "One simple step" entry: AudioWorklet + WASM jam core + WebTransport
  datagrams vs tuned WebRTC vs native helper launched from the browser
- Measure OS/browser audio buffer latency per platform (Windows, macOS,
  Android, iOS)
- Decide the entry form from evidence; no winner before evidence

## ECHLUB-JAM-HUMAN-TRIAL-01

- 2 → 4 players, rock; calibrate musician-sim parameters and verdict
  thresholds against real sessions

## Parked

- ECHLUB-COMPOSITION-INTENT-01 (two-browser intent exchange, ghost preview):
  not on the current critical path.
- ECHLUB-PERFORMANCE-BASELINE-01: implemented; the WebRTC lab remains the
  browser baseline.
