# Live Two-Peer Known Issues

- Clock offset is an estimate only; route asymmetry and scheduling affect it.
- Browser stats are implementation-reported; not ground-truth network measurement.
- First baseline prefers wired/non-Bluetooth audio; Bluetooth should be recorded as a limitation.
- Grok routing remains unverified from prior multitask probe (`MODEL_IDENTITY_UNVERIFIABLE`).
- Two-device LAN evidence requires owner manual run; CI does not request microphone access.
