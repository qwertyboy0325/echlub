# Product Vision

## Current goal (owner decision, 2026-10-02)

**Let musicians in different parts of Taiwan play together in real time.**

- First target: rock bands, 2–4 players; the final trial for this phase is a
  4-piece (drums, bass, guitar, vocals).
- Entry must feel like **one simple step**. Opening a browser tab is the
  preferred form, but the owner has explicitly allowed overturning
  browser-only if evidence shows it cannot meet the latency budget (see
  ADR-0006).
- Work goes straight to a prototype, backed by simulated musicians so ideas
  can be exercised before real players are recruited.
- Target terminal and first experiment condition: **every player on an
  iPhone with a native app and wired monitoring** (ADR-0007).
- Current conclusions and the catalogue of network and non-network latency
  strategies: `jam-latency-conclusions.md`. Interval (bar-delay) mode is
  deferred.

This supersedes the 2026-07-28 posture of "no accepted product thesis".
Collaborative *composition* (shared arrangement editing) remains out of the
headline; this goal is about playing together live.

## Why this goal is perceptible

The earlier collaboration demo failed because a cold viewer could not
perceive collaboration. Real-time playing does not have that problem: a
drummer in Taipei and a bassist in Kaohsiung either lock in or they do not,
and anyone listening can tell. Success criteria are therefore musical
("the band stays together") plus measured latency.

## Planes

- **Performance plane (primary):** audio capture, transport, jitter
  buffering, mixing, timing, measurement.
- **Session fabric:** rooms, identity, capability negotiation; needed for
  "one simple step" entry.
- **Composition plane (parked):** deterministic semantic operations remain in
  the repository but are not on the critical path.

## Claim discipline

- Simulation outputs are labelled `Simulation` and rest on **assumed**
  network, device, and musician parameters. They guide design; they are not
  evidence about real networks or people.
- No latency claim without measured evidence. No final transport selection
  until the bake-off in the roadmap produces evidence.
- No production-readiness claim.
