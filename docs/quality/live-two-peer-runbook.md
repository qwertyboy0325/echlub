# Live Two-Peer Runbook

> **Status: SUSPENDED** pending owner manual two-device observation. Automated correction package (ECHLUB-PERFORMANCE-LAB-REAUDIT-CORRECTION-01) hardens harness and validators; live endpoint evidence still requires owner run per stop gate `OWNER_LIVE_RUN_REQUIRED`.

## Host device

```bash
git fetch origin
git switch work/live-two-peer-observation-01
git pull --ff-only
corepack pnpm install --frozen-lockfile
python3 scripts/run-live-two-peer.py host --bind 0.0.0.0:8080
```

Open `http://localhost:5173` on the host device.

## Peer device

```bash
git fetch origin
git switch --detach origin/work/live-two-peer-observation-01
corepack pnpm install --frozen-lockfile
python3 scripts/run-live-two-peer.py peer
```

Open `http://localhost:5173` on the peer device. Enter `ws://<HOST_LAN_IP>:8080/v1/signaling/ws` in the UI (do not commit the LAN IP).

## Observation procedure

1. Two physically distinct devices on the same LAN, same Git commit.
2. Chrome or Edge preferred; headphones on both devices.
3. Generate/copy the same session correlation ID on both devices.
4. Assign `peer_a` on host, `peer_b` on peer; same capture profile.
5. Acknowledge headphones, prepare, enable microphone, connect.
6. Wait for **Ready To Observe** (clock warmup probes and stats preflight must succeed).
7. Run 60-second observation on both.
8. Export **Finalized Endpoint** artifacts from each device (not diagnostic draft).
9. Copy to `.local/live-observation-import/peer-a.json` and `peer-b.json` (gitignored; never commit).

Do not expose port 8080 to the public Internet.
