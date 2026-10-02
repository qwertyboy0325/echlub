# ECHLUB-REALTIME-JAM-PROTOTYPE-01

First prototype for real-time jamming across Taiwan (ADR-0006), with
simulated musicians.

## Status

```yaml
status: implemented (prototype + simulation)
human_trial: not started
field_network_measurement: not started
latency_claim: none
```

## Delivered

- `crates/echlub-jam`: wire format, jitter buffer, mix-minus, impairment
  model, Taiwan site/access/endpoint presets (all `Assumed`), latency budget
  for `mix` and `forward` topologies, probe clicks + onset detection,
  deterministic pipeline simulator, WAV encoder. Builds for
  `wasm32-unknown-unknown`.
- `crates/echlub-musician-sim`: rock four-piece timing model (phase/period
  correction, anticipation, tempo memory, count-in), ensemble metrics and
  verdicts, latency sweep, procedural synth and "what each player hears"
  rendering.
- `apps/jam-relay`: UDP relay prototype (`--mode mix|forward`).
- `apps/jam-lab`: `scenario`, `budget`, `sweep`, `bots` commands.
- `scripts/run-jam-simulation.py` and outputs under
  `evidence/realtime-jam-simulation/2026-10-02/`.
- Tests: unit tests per module, pipeline determinism/topology tests, ensemble
  behaviour tests, and localhost UDP integration tests (relay + bots, both
  topologies).

## Findings (simulation, assumed inputs; not evidence)

1. The simulated band stays tight to ~15–20 ms uniform one-way latency,
   playable to ~30 ms, then tempo drags. Qualitatively consistent with
   published delayed-ensemble studies, but parameters are uncalibrated.
2. With audio interfaces and wired fibre at all four sites, a Taiwan 4-piece
   lands at ~18–23 ms (forward) / ~20–31 ms (mix): playable in simulation.
3. One Wi-Fi or 4G player pushes their paths to 50–100+ ms and the band out
   of the playable region. Wired connections matter more than distance.
4. Stock WebRTC audio settings (assumed 20 ms Opus frames, voice processing,
   jitter floor) land around 110 ms: far outside the budget. A tuned
   browser path (AudioWorklet + datagrams) is assumed at ~37–50 ms, still
   above the comfortable region unless OS audio buffers prove smaller.
5. Jitter buffers are the largest controllable cost; `forward` saves one per
   path.

## Mobile-player study (simulation, assumed inputs; not evidence)

`jam-lab mobile-study --role vocals|drums`: a wired 4-piece where one player
is on 4G/5G, either directly on a phone (`phone-app` device), via a phone
hotspot over Wi-Fi or USB to a laptop with an audio interface, or on 5G SA.
Remedies explored: jitter-buffer coverage (99/95/90%), packet redundancy
(each frame sent twice), and three band arrangements (`Normal`, `Follower`,
`FollowerClickAhead`). Outputs: `evidence/realtime-jam-simulation/2026-10-02/mobile-study-*`.

1. A hotspot does not fix the radio link. Its value is letting the player use
   a laptop and audio interface instead of the phone's audio stack. USB
   tethering beats hotspot Wi-Fi (no phone access-point jitter).
2. Packet redundancy is the largest single remedy for 4G: on a 4G USB hotspot
   at 99% coverage the mobile path drops from ~108 ms to ~71 ms with no
   audible dropouts, at 2× bandwidth (≈1.8 Mbit/s up, ≈5.5 Mbit/s down for a
   4-piece in `forward`, raw PCM).
3. Lowering coverage to 90–95% saves another ~5–11 ms for occasional
   audible gaps.
4. 4G never reaches "whole band playable". The wired players stay fine, but
   a 4G player who follows what they hear sounds ~60–130 ms late to the
   band (about one round trip).
5. `Follower` (band ignores the 4G player's timing) cuts tempo drag roughly
   in half, more so when the drummer is on 4G, but makes the 4G player sound
   later.
6. `FollowerClickAhead` (synced click played early for the 4G player) puts
   the 4G player within ~±25 ms of the beat for the band and keeps tempo
   stable, but the 4G player hears the band ~140–170 ms late and must play
   to the click. Human feasibility is unverified.
7. 5G via USB hotspot or 5G SA with redundancy reaches "whole band playable"
   in simulation (mobile paths ~26–42 ms). Real Taiwan 5G SA availability
   and latency must be measured.

New core pieces: `Concealment::RepeatFade`, audible-dropout counting
(gaps ≥ 8 ms), jitter quantile/coverage sizing, `PeerSetup::with_redundancy`,
per-pair heard-offset matrices, `!follow` / `!ahead` player syntax.

## Stop gate

Before any latency claim or transport decision:

- replace assumed profiles with Taiwan field measurements
  (`ECHLUB-TAIWAN-NETWORK-BASELINE-01`);
- measure real device and browser audio latency with a hardware loopback;
- run a human trial with at least two real players.

## Forbidden

- Claiming measured latency from simulation output.
- Selecting a final transport or topology without measured evidence.
- API-backed models for musician simulation.
