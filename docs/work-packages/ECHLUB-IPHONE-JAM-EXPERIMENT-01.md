# ECHLUB-IPHONE-JAM-EXPERIMENT-01

Experiment protocol for ADR-0007: all players on iPhone, native app, wired
monitoring. Not started; requires the iOS client
(`ECHLUB-PHONE-TERMINAL-01`) and a relay in central Taiwan.

## Fixed conditions

| Item | Condition |
| --- | --- |
| Terminal | iPhone 15 or later (USB-C), native EchLub app |
| Audio session | `.playAndRecord`, `.measurement` mode, 48 kHz, IO buffer 256 frames (then 128) |
| Monitoring | Wired headphones via USB-C (closed-back for drums). No Bluetooth, no speaker |
| Input | Vocals: headset or built-in mic. Guitar/bass: USB-C class-compliant interface. Drums: built-in mic near the kit |
| Relay | Central Taiwan, `forward` topology |
| Transport defaults | 2 copies per frame, jitter coverage 95% |
| Music | Rock, 4/4, 120 BPM, I–V–vi–IV riff plus a song with shared stops |

Hardware per player: iPhone, wired headphones (USB-C or adapter). Ethernet
adapter only for the reference runs. Guitar/bass need a USB-C instrument
interface (which can also drive the headphones).

User-facing network condition (ADR-0007 amendment): tuned Wi-Fi — 5 GHz
Wi-Fi 6 near the router, voice-priority marking, AirDrop/Handoff off,
Low Power Mode off, 128-frame buffer.

## Variables, in order

1. **Network:** Ethernet adapter (reference) → tuned Wi-Fi (target) →
   typical home Wi-Fi → 5G SA.
2. **IO buffer:** 256 → 128 frames.
3. **Band size:** 2 → 3 → 4 players.
4. **Distance:** same city → north–south.

## Stages and gates

| Stage | Setup | Gate to continue |
| --- | --- | --- |
| 0 | Each iPhone alone: loopback self-test (round-trip audio latency) | Measured device latency recorded; Bluetooth refused |
| 1 | Duo, same room/LAN | App works end to end; probe latency matches budget within ±5 ms |
| 2 | Duo, same city, Ethernet | Players rate "can play together" |
| 3 | Duo, Taipei ↔ Kaohsiung, Ethernet | Same rating as stage 2 |
| 4 | Rock 4-piece across Taiwan, Ethernet | Band completes a song without falling apart |
| 5 | Network variants (Wi-Fi, 5G SA) | Compare ratings and measurements with predictions |

## Predictions to test (simulation, assumed inputs)

From `evidence/realtime-jam-simulation/2026-10-02/iphone-target/`:

| Condition | Predicted mouth-to-ear (ms) | Predicted |
| --- | --- | --- |
| Duo same city, Ethernet, 256 | ~25 | playable |
| Duo Taipei–Kaohsiung, Ethernet, 256 | ~27 | playable |
| 4-piece, Ethernet, 256 | ~25–27 | playable |
| 4-piece, Ethernet, 128 | ~19–22 | playable, partly tight |
| 4-piece, 3 Ethernet + vocals on Wi-Fi | ~25–35 | playable |
| 4-piece, 3 Ethernet + drums on Wi-Fi | ~25–35 | borderline |
| 4-piece, 2 on Wi-Fi | ~25–43 | not playable |
| 4-piece, all tuned Wi-Fi, 128 | ~25–27 | playable (**target**) |
| 4-piece, all tuned Wi-Fi, 256 | ~30–33 | playable |
| 4-piece, all typical Wi-Fi, 256 | ~35–43 | not playable |
| 4-piece, all 5G SA | ~41–46 | not playable |
| 4-piece, Ethernet, Bluetooth monitoring | ~170 | not playable |

A prediction is *confirmed* when measured latency is within ±5 ms and the
players' rating matches; otherwise the assumed profile is corrected and the
musician model recalibrated.

## Measurements per session

- In-signal probe latency per pair (p50/p95), concealed frames, audible
  dropouts per minute.
- Device round-trip latency (stage 0) per iPhone.
- Tempo drift over the song (from recorded onsets).
- Player ratings per pair: 1 (impossible) to 5 (like the same room), plus
  free comments.

## Forbidden

- Claiming results beyond the tested devices, networks, and players.
- Mixing Android or browser terminals into this experiment.
