# Session Handoff — 2026-10-02

For a session continuing this work on the owner's machine. Branch:
`claude/happy-goldberg-kiba8e` (not merged; no PR).

## Owner decisions so far

1. Goal: real-time jamming across Taiwan, rock, 2–4 players, final trial a
   4-piece (ADR-0006, `docs/product/vision.md`).
2. Prototype first, with simulated musicians (rule-based, no API models).
3. Target terminal / experiment condition: **all iPhone, native app, wired
   headphones** (ADR-0007).
4. No network cable on the phone for users: tuned Wi-Fi is the default;
   Ethernet adapter only as an experiment reference (ADR-0007 amendment).
5. Interval ("one bar late") mode is deferred.
6. No field-test environment yet; every number so far is simulation with
   assumed profiles.

## What exists

- `crates/echlub-jam`: platform-neutral jam core (must keep building for
  `wasm32-unknown-unknown`).
- `crates/echlub-musician-sim`: simulated rock musicians.
- `apps/jam-relay`: UDP relay (`mix` / `forward`).
- `apps/jam-lab`: `scenario`, `budget`, `sweep`, `mobile-study`, `bots`.
- `scripts/run-jam-simulation.py`: regenerates
  `evidence/realtime-jam-simulation/<label>/`.
- Key docs: `docs/product/jam-latency-conclusions.md`,
  `docs/architecture/realtime-jam.md`,
  `docs/work-packages/ECHLUB-REALTIME-JAM-PROTOTYPE-01.md`,
  `docs/work-packages/ECHLUB-IPHONE-JAM-EXPERIMENT-01.md`,
  `docs/roadmap/next-work-packages.md`.

## Verify locally

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo build -p echlub-jam -p echlub-musician-sim --target wasm32-unknown-unknown
python3 scripts/run-jam-simulation.py --label <date>   # optional, ~1 min
```

## Suggested next steps

1. `ECHLUB-PHONE-TERMINAL-01` (iOS first; needs a Mac with Xcode): wrap
   `echlub-jam` for iOS (static library + C/Swift bindings), Core Audio
   engine in `.measurement` mode, 128-frame buffer, wired-headphone check,
   on-device loopback latency test, UDP client for `jam-relay --mode forward`
   with 2 copies per frame and voice-priority service class.
2. `ECHLUB-JAM-DEVICE-SELF-CHECK-01` with Wi-Fi coaching.
3. Run the relay on a small VM in central Taiwan for the first duo test.

## Known gaps

- Real-time bots do not send redundant copies yet (simulator does).
- No clock-drift handling, no codec, fixed-depth jitter buffers.
- Musician model thresholds and all device/network profiles are uncalibrated.
