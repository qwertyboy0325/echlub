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
.local/live-automation/<automation-run-id>/
  scenarios/
    peer-a-first/
      raw/
        peer-a.json
        peer-b.json
      paired/
        peer-a.validated.json
        peer-b.validated.json
        peer-a.summary.json
        peer-b.summary.json
        pair-summary.json
        report.md
        artifact-manifest.json
      diagnostics/
        peer-a-screenshot.png
        peer-b-screenshot.png
        peer-a-trace.zip
        peer-b-trace.zip
        browser-console.json
        process-log-reference.json
    peer-b-first/
      raw/
      paired/
      diagnostics/
  automation-report.json
  diagnostics/
    process-log.txt
```

Each connect-order scenario receives its own isolated directory tree. Validation, pairing, and directory verification run **inside** each scenario before it may report `PASS`. Overall automation `PASS` requires both scenarios to pass independently.

Timing fields in `automation-report.json`:

| Field | Meaning |
|-------|---------|
| `uiClickDispatchDeltaMs` | Playwright click dispatch skew between peers |
| `actualEndpointStartDeltaMs` | Absolute difference of endpoint `startedAtUtc` values (must be ≤ 2000 ms) |

## Scenarios

The harness runs two connection-order scenarios on one host:

1. Peer A connects, then Peer B (`scenarios/peer-a-first/`)
2. Peer B connects, then Peer A (`scenarios/peer-b-first/`)

Each scenario requires both peers to reach **Ready To Observe**, dispatch observation start within a 2000 ms UI window, complete the full 60-second production observation, export finalized endpoints, pass independent validate/pair/verify checks, and retain its own diagnostics.

## Failure diagnostics

On failure, inspect:

- `.local/live-automation/<run-id>/automation-report.json`
- `scenarios/<scenario>/diagnostics/browser-console.json`
- `scenarios/<scenario>/diagnostics/peer-a-screenshot.png` and `peer-b-screenshot.png`
- `scenarios/<scenario>/diagnostics/peer-a-trace.zip` and `peer-b-trace.zip`
- `diagnostics/process-log.txt`
- `scenarios/<scenario>/diagnostics/process-log-reference.json`

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
