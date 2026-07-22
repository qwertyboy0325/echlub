# Single-Host Dual-Browser Live Automation

## Purpose

This harness runs a **readiness-only** automated check of the supported EchLub live two-peer browser flow on one machine. It launches two isolated Chromium processes with deterministic fake microphone WAV inputs, drives the production UI through connect and 60-second observation, downloads finalized endpoints, and runs the production validation and pairing pipeline into a temporary directory.

**PASS does not mean physical two-device evidence is accepted.** The automation classification is `automated_single_host_dual_browser` with `evidenceAuthority: readiness_only`.

## Command

```bash
corepack pnpm install
corepack pnpm exec playwright install chromium
corepack pnpm live:e2e:single-host
```

Requirements before running:

- clean Git working tree;
- local `HEAD` matches the commit under test;
- Rust toolchain and Node 20+ available;
- headed Chromium supported locally (the harness sets `headless: false`).

## Local dependencies

- Playwright bundled Chromium (`playwright install chromium`)
- EchLub control-plane and web dev servers (started automatically)
- `cargo run -p echlub-performance-report` for validate/pair/verify commands

## Fake microphone fixtures

Deterministic PCM WAV files are generated under `tools/live-dual-browser-harness/fixtures/` when missing:

| Peer | Signal |
|------|--------|
| Peer A | recurring 697 Hz tone pattern |
| Peer B | recurring 1209 Hz tone pattern |

Format: mono, 48 kHz, 16-bit PCM, 90 seconds. These signals are for debugging only and are **not** acoustic measurements.

## Output location

All automation artifacts are written to:

```text
.local/live-automation/<automation-run-id>/
```

This directory is gitignored. The harness fails if its output path resolves under `.local/live-observation-import/`.

Layout:

```text
raw/peer-a.json
raw/peer-b.json
paired/…
diagnostics/…
automation-report.json
```

## Scenarios

The harness runs two connection-order scenarios on one host:

1. Peer A connects, then Peer B
2. Peer B connects, then Peer A

Each scenario requires both peers to reach **Ready To Observe**, start observation within a 2000 ms window, complete the full 60-second production observation, and export finalized endpoints through the UI.

## Failure diagnostics

On failure, inspect:

- `.local/live-automation/<run-id>/automation-report.json`
- `diagnostics/browser-console.json`
- `diagnostics/peer-a-screenshot.png` and `peer-b-screenshot.png`
- `diagnostics/playwright-trace.zip`
- `diagnostics/process-log.txt`

Captured fields include UI phase, connection/ICE/DataChannel state, sample/probe counts, visible errors, browser console output, and server logs.

## Evidence boundary

| Claim | Automation |
|-------|------------|
| Supported UI flow works on one host | Yes, when result is PASS |
| Physical two-device observation | **No** |
| Real LAN path proven | **No** |
| Independent hardware clocks proven | **No** |
| Acoustic latency measured | **No** |
| Owner import directory populated | **No** |

The headphones acknowledgement in the UI is a **test acknowledgement** only. It does not claim physical headphones exist.

## Relationship to physical two-device evidence

`OWNER_LIVE_RUN_REQUIRED` remains separate. Owner manual imports belong only in `.local/live-observation-import/` and must come from physically distinct devices. Do not copy automation exports into the owner import directory or classify automation output as physical evidence.

## CI

CI runs harness **unit tests** and path/evidence-boundary checks. The headed end-to-end command is intended for local owner/developer runs when Chromium is available; a skipped local E2E run must not be reported as PASS.
