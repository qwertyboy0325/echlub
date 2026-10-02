# Real-Time Jam Architecture

## Components

| Component | Kind | Role |
| --- | --- | --- |
| `crates/echlub-jam` | platform-neutral lib | Wire format, jitter buffer, mix-minus, impairment model, latency budget, probe clicks + onset detection, pipeline simulator, WAV encoder |
| `crates/echlub-musician-sim` | platform-neutral lib | Simulated rock musicians: timing model, ensemble metrics, verdicts, procedural synth |
| `apps/jam-relay` | native bin + lib | UDP relay prototype, `mix` or `forward` mode |
| `apps/jam-lab` | native bin + lib | `scenario`, `budget`, `sweep`, and `bots` (real-UDP bot musicians) |
| `scripts/run-jam-simulation.py` | script | Regenerates `evidence/realtime-jam-simulation/<label>/` |

Dependencies: `echlub-jam` ← `echlub-musician-sim` ← `jam-lab`;
`echlub-jam` ← `jam-relay` ← `jam-lab`. Nothing here depends on the
composition crates.

## Audio path

- 48 kHz mono, 128-sample frames (2.67 ms), 16-bit PCM, no codec yet.
- Packet: `"EJ"` magic, version, kind (hello/bye/audio), peer id, sequence,
  sender sample time, PCM.
- **Mix topology:** relay keeps one jitter buffer per peer, ticks on its own
  frame clock, sends each peer a mix-minus (everyone but itself).
- **Forward topology:** relay forwards each packet on arrival; each client
  keeps one jitter buffer per source and mixes locally. One jitter buffer
  per path instead of two, at the cost of N−1 downstreams (≈ 3 × 0.8 Mbit/s
  for a 4-piece with raw PCM).
- Musicians always hear themselves locally (zero-latency monitoring).

## One-way latency budget (mouth to ear)

capture buffer + ADC + frame packetization + uplink + (relay jitter buffer +
relay tick, mix only) + downlink + client jitter buffer + client tick +
receive processing + playback buffer + DAC.

`jam-lab budget` prints this for any scenario. The pipeline simulator
measures the network-and-buffer part from probe clicks in the played-out
audio and adds the device part analytically.

## Simulated musicians

Per beat, each player updates its next onset and its tempo from the
asynchrony between its own onset and the weighted onsets it *heard*
(others delayed by the latency matrix), with:

- phase correction `alpha` and period correction `beta`;
- **anticipation**: players habitually aim a few ms ahead of what they hear;
- **tempo memory** `gamma`: a pull back toward the intended tempo;
- timekeeper and motor noise;
- optional conscious latency compensation;
- a drummer count-in heard through the network.

Listening weights favour the drummer, then bass. Verdicts (`Tight`,
`Playable`, `Struggling`) use tempo drift and worst heard RMS asynchrony with
**hypothesised** thresholds. All parameters are assumptions to calibrate with
real players.

## Known limits

- No clock drift between devices; no packet-loss concealment beyond silence.
- No codec (Opus) yet; bandwidth is raw PCM.
- Fixed-depth jitter buffers chosen from assumed p99 jitter; no adaptation.
- Real-time bots share one process clock; their latency numbers describe the
  host machine plus injected impairment only.
- Socket read timeouts are too coarse on some platforms (≈8 ms in this
  sandbox), so the relay and bots poll non-blocking sockets with short
  sleeps. A product client should be clocked by the audio device callback.
