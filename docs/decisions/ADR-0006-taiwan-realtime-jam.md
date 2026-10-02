# ADR-0006 Taiwan Real-Time Jam Target

## Status

Accepted by owner, 2026-10-02.

## Context

The owner revived the remote-playing direction: musicians across Taiwan,
2–4 players (final trial: rock 4-piece), with entry that feels like one simple
step. A browser was the preferred client, but the owner accepted that this may
be overturned by evidence.

Rhythmic ensemble playing degrades as one-way mouth-to-ear delay grows;
published delayed-ensemble studies commonly place a comfortable region around
or below ~20–30 ms one-way. Taiwan's geography keeps backbone propagation to a
few milliseconds, so the budget is dominated by endpoints (audio buffers,
codec framing, jitter buffers) rather than distance.

## Decision

1. **Target:** real-time rock jamming for 2–4 players across Taiwan via a relay
   in central Taiwan.
2. **Platform-neutral jam core in Rust** (`echlub-jam`): packet format, jitter
   buffer, mix-minus, latency budget, impairment model, pipeline simulator.
   No sockets, threads, audio devices, or browser APIs, so the same code can
   back a native client, the relay, and a WASM AudioWorklet. It must keep
   building for `wasm32-unknown-unknown`.
3. **Simulated musicians** (`echlub-musician-sim`) are rule-based timing
   models (phase/period correction with anticipation and tempo memory), not
   language models. They exist to exercise the system before human trials.
4. **Topology is an open experiment**, not a decision: `mix` (relay buffers and
   mixes; two jitter buffers per path) vs `forward` (relay forwards; clients
   buffer per source and mix; one jitter buffer per path). Simulation favours
   `forward` by a few ms; human trials decide.
5. **"One simple step" is the requirement; "browser-only" is a preference.**
   Candidates, to be decided by measured evidence:
   - browser with AudioWorklet + WASM jam core + datagram transport
     (WebTransport), bypassing WebRTC's audio jitter buffer and 20 ms Opus
     frames;
   - a small native helper launched from the browser (one install, then
     one click per session);
   - stock WebRTC audio, kept only as a baseline.

## Consequences

- ADR-0003 (web-first, not browser-locked) and ADR-0004 (transport-independent
  protocol) still hold; this ADR narrows their purpose.
- Assumed profiles in `crates/echlub-jam/src/profiles.rs` must be replaced by
  field measurements before any latency claim.
- Simulations so far (assumed inputs) indicate stock WebRTC audio is far
  outside the budget and that wired connections and audio interfaces matter
  more than distance. These are design signals, not evidence.
