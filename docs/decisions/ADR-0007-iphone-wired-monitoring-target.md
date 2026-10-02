# ADR-0007 All-iPhone With Wired Monitoring as Target and Experiment Condition

## Status

Accepted by owner, 2026-10-02.

## Decision

The target terminal and the first experiment condition for real-time jamming
(ADR-0006) is: **every player on an iPhone, running a native app, monitoring
through wired headphones.**

- Native iOS app on Core Audio (`AVAudioSession` `.playAndRecord`,
  `.measurement` mode: no voice processing), 48 kHz, 128- or 256-frame IO
  buffer.
- Wired monitoring only. Bluetooth output is refused or blocked by the app.
- Network is an experiment variable, ordered by expected viability:
  USB-C Ethernet adapter > Wi-Fi > 5G SA > 5G NSA > 4G.

## Rationale

- One device family removes Android audio-path variance and gives a single,
  measurable audio stack.
- A native app is required; the mobile-browser path is assumed too slow.
- Simulation (assumed inputs) puts an all-iPhone rock 4-piece across Taiwan
  at ~25–27 ms on Ethernet (playable), ~19–22 ms with a 128-frame buffer
  (partly tight), and ~35–43 ms on Wi-Fi (not playable); Bluetooth
  monitoring alone pushes paths to ~170 ms.

## Consequences

- `ECHLUB-PHONE-TERMINAL-01` becomes iOS-first; Android is later.
- iPhones with USB-C (iPhone 15 and later) are preferred for the experiment:
  one hub can carry Ethernet, headphones, and an instrument interface.
- The `phone-ios-*` endpoint profiles are assumptions until replaced by
  on-device loopback measurement.
