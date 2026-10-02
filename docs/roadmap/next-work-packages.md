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
  Tainan/Kaohsiung, Hualien) across ISPs and access types, including 4G,
  5G NSA, 5G SA where available, and phone hotspot over Wi-Fi vs USB
- Measure loss burstiness (it decides whether simple packet duplication is
  enough on cellular)
- Replace `Assumed` profiles in `echlub-jam` with measured ones
- Gate: p95 one-way to relay on wired fibre

## ECHLUB-BROWSER-JAM-PATH-01

- "One simple step" entry: AudioWorklet + WASM jam core + WebTransport
  datagrams vs tuned WebRTC vs native helper launched from the browser
- Measure OS/browser audio buffer latency per platform (Windows, macOS,
  Android, iOS)
- Decide the entry form from evidence; no winner before evidence

## ECHLUB-JAM-MOBILE-REMEDIES-01

- Redundant copies and coverage control in the real relay/bots and client
- Synced click track with per-player advance (`FollowerClickAhead`); test
  whether a 4G player can play to an early click while hearing the band late
- Better packet-loss concealment than repeat-and-fade

## ECHLUB-PHONE-TERMINAL-01

- Native phone client (iOS Core Audio; Android AAudio/Oboe) on the jam core
- On-device loopback latency measurement; detect Android low-latency
  support; refuse or warn on Bluetooth output
- Optional USB-C Ethernet and USB audio interface support
- Replace the assumed `phone-*` endpoint profiles with measurements

## ECHLUB-JAM-DEVICE-SELF-CHECK-01

- Pre-session check: device round-trip latency, Bluetooth detection,
  network type and jitter, suggested role

## ECHLUB-JAM-SHARED-CLICK-01

- Clock-synced click/backing track; per-player advance for slow links

## ECHLUB-JAM-HUMAN-TRIAL-01

- 2 → 4 players, rock; calibrate musician-sim parameters and verdict
  thresholds against real sessions

## Deferred by owner

- ECHLUB-JAM-INTERVAL-MODE-01: NINJAM-style interval mode (others heard
  exactly one bar/phrase late). Works on any network, including 4G and
  browsers; revisit after the real-time path is evaluated.

## Parked

- ECHLUB-COMPOSITION-INTENT-01 (two-browser intent exchange, ghost preview):
  not on the current critical path.
- ECHLUB-PERFORMANCE-BASELINE-01: implemented; the WebRTC lab remains the
  browser baseline.
