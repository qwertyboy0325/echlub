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
