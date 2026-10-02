# Real-Time Jam: Conclusions and Latency Strategies

Status as of 2026-10-02. **Every number here comes from simulation with
assumed network, device, and musician parameters.** No field measurement
exists yet; the owner has no test environment for that at present.

## Converged conclusions

1. **Distance inside Taiwan is not the bottleneck; endpoints and access are.**
   Backbone propagation is a few ms. Audio buffers, jitter buffers, and
   Wi-Fi/cellular jitter dominate.
2. **Budget for rock: roughly ≤ 25–30 ms one-way mouth-to-ear.** The simulated
   band is tight to ~20 ms, playable to ~30 ms, then tempo drags. Thresholds
   are hypotheses until calibrated with real players.
3. **Wired network + audio interface (or a low-latency phone audio path) +
   a low-latency client reaches the budget** for a 4-piece across Taiwan.
4. **Stock browser WebRTC audio (~110 ms) is unsuitable.** "One simple step"
   must come from a custom browser audio path (unverified) or a browser
   lobby plus a small native app.
5. **Wi-Fi and 4G break the budget; 5G may not.** 4G with every remedy still
   leaves that player about a round trip behind the band. 5G via USB hotspot
   or 5G SA, with packet redundancy, is playable in simulation.
6. **Network remedies that help:** `forward` relay topology (one jitter buffer
   per path), sending each frame twice, tuning jitter-buffer coverage.
7. **Unverified:** real Taiwan network figures, real device and phone audio
   latency, real musicians' tolerance.

## Non-network strategies

Status: **sim** = modelled in `echlub-musician-sim`/`jam-lab`; **doc** = documented
only; **deferred** = owner postponed.

### A. Endpoint setup (cheapest, largest effect)

| Strategy | Status | Note |
| --- | --- | --- |
| No Bluetooth audio | doc | Bluetooth alone commonly adds ≥ 100 ms |
| Wired headphones, no speakers | doc | Avoids echo and echo-cancellation latency |
| Audio interface / low-latency phone audio path, small buffers | sim | `native-interface`, `phone-ios-app`, `phone-android-low-latency`, `phone-interface` |
| Disable OS voice processing (noise suppression, AGC) | doc | Each stage adds latency |
| Pre-session device self-check | doc | Measure device latency, flag Bluetooth/Wi-Fi before joining |

### B. How the band plays

| Strategy | Status | Note |
| --- | --- | --- |
| Lowest-latency players hold the rhythm section | sim | A drummer on a slow link drags the band far more than a vocalist |
| `Follower` arrangement for a slow player | sim | Halves tempo drag; slow player sounds later |
| Shared synced click, played early for slow players (`FollowerClickAhead`) | sim | Slow player lands on the beat for the band but hears the band ~1 round trip late; human feasibility unknown |
| Hear yourself through the server (Jamulus-style self-delay) | doc | Everyone shares one delay; not yet modelled |
| Repertoire: slower tempos, sustained parts, repeating riffs | sim | 40 ms drags tempo ~2% at 80 BPM vs ~4% at 160 BPM; perceived lateness in ms does not shrink |

### C. Redefine "real time"

| Strategy | Status | Note |
| --- | --- | --- |
| Interval mode: everyone hears others exactly one bar/phrase late (NINJAM-style) | **deferred** | Immune to network latency, works on any link and in a browser; cannot do shared stops/hits |
| Chain mode: each player hears only upstream players | doc | Good for rehearsal recording or streaming |
| Asynchronous layered recording | doc | Not playing together |

### D. What is sent

| Strategy | Status | Note |
| --- | --- | --- |
| MIDI with timestamps for electronic instruments | doc | Tiny packets; scheduled playout removes jitter, not latency |
| Predicting other players' timing | doc | Research only |

### E. Product and community

| Strategy | Status | Note |
| --- | --- | --- |
| Match players by compatible latency | doc | |
| Show a latency score and suggested roles before a session | doc | |
| Choose relay per group location | doc | |

## Target condition: all iPhone + wired monitoring (ADR-0007)

| All-iPhone 4-piece | Paths (ms) | Band playable |
| --- | --- | --- |
| Ethernet adapter, 256-frame buffer | ~25–27 | yes |
| Ethernet adapter, 128-frame buffer | ~19–22 | yes, partly tight |
| 3 Ethernet + vocals on Wi-Fi or 5G SA | ~25–35 | yes |
| 3 Ethernet + drums on Wi-Fi | ~25–35 | borderline |
| All Wi-Fi | ~35–43 | no |
| All 5G SA / 5G / 4G | ~41–46 / ~59–70 / ~97–113 | no |
| Ethernet, Bluetooth monitoring | ~170 | no |

Same-city vs Taipei–Kaohsiung differs by only ~3 ms. Protocol:
`docs/work-packages/ECHLUB-IPHONE-JAM-EXPERIMENT-01.md`.

## Phones as terminals (planned for some experiments)

Simulated with a wired 4-piece where the vocalist is on a phone, or all four
on phones (`evidence/realtime-jam-simulation/2026-10-02/phone-terminals/`;
forward topology, 2 copies per frame, 95% coverage; "wired" for a phone
means a USB-C Ethernet adapter):

| Phone setup (one player) | Worst path (ms) | Band playable |
| --- | --- | --- |
| USB audio interface, wired | ~20 | yes |
| iOS app, wired / Wi-Fi or 5G SA | ~23 / ~31 | yes |
| Android low-latency (AAudio/Oboe), wired / Wi-Fi or 5G SA | ~27 / ~35 | yes |
| Android without a fast audio path, wired / Wi-Fi | ~48 / ~56 | borderline / no |
| Mobile browser, wired / Wi-Fi | ~49 / ~57 | borderline / no |

| All four on phones | Paths (ms) | Band playable |
| --- | --- | --- |
| iOS app, wired | ~25–27 | yes |
| iOS app, Wi-Fi | ~35–43 | no |
| Android low-latency, wired | ~29–32 | yes |
| Android generic, wired | ~64–67 | no |

Implications for phone experiments:

- A **native app** matters more than the phone itself; the mobile browser
  path is assumed too slow.
- **Android varies by device:** only devices with a low-latency audio path
  are viable; the app must detect this.
- **Wired headset or USB audio is required;** Bluetooth is excluded.
- A **USB-C Ethernet adapter** makes a phone behave like a wired laptop on
  the network side; Wi-Fi/5G SA costs about 8 ms more per path in this model.
- Phone profiles are assumptions to be replaced by on-device loopback
  measurement in the first phone experiment.
