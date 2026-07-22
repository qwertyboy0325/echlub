# Live Two-Peer Runbook

> **Status: SUSPENDED** pending owner external review before manual two-device observation. Automated correction packages harden harness, validators, and setup flow; live endpoint evidence still requires owner run per stop gate `OWNER_LIVE_RUN_REQUIRED`.

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

## Shared session setup

1. On **Peer A**, click **Generate** to create a session correlation ID (or keep the generated value).
2. On **Peer A**, click **Copy** to copy the 32-character correlation ID.
3. On **Peer B**, paste the same ID into the editable **Session correlation ID** field.
4. On both devices, verify the exact same 32-character hexadecimal ID is displayed before **Prepare**.
5. Assign **Peer A** on one device and **Peer B** on the other; choose the same capture profile and signaling URL on both.
6. Click **Prepare** on each device. After Prepare, correlation ID, role, signaling URL, and capture profile are frozen until **Reset**.
7. Either device may connect first; Peer A negotiation starts once both peers are present regardless of join order.

## Observation procedure

1. Two physically distinct devices on the same LAN, same Git commit.
2. Chrome or Edge preferred; headphones on both devices.
3. Complete the shared session setup steps above.
4. Acknowledge headphones, enable microphone, connect on both devices.
5. Wait for **Ready To Observe** (clock warmup probes and stats preflight must succeed).
6. Run 60-second observation on both.
7. Export **Finalized Endpoint** artifacts from each device (not diagnostic draft).
8. Copy to `.local/live-observation-import/peer-a.json` and `peer-b.json` (gitignored; never commit).

Do not expose port 8080 to the public Internet.

Manual two-device run remains **not authorized** in this document until owner external review completes.
