# Live Two-Peer Observation Evidence

Validated paired browser observations are stored under `evidence/live-two-peer/<pair-id>/` after an owner-assisted two-device LAN run.

## Required artifacts (after pairing)

- `peer-a.validated.json`
- `peer-b.validated.json`
- `peer-a.summary.json`
- `peer-b.summary.json`
- `pair-summary.json`
- `report.md`
- `artifact-manifest.json`

## Claims boundary

- Exploratory non-authoritative browser network observation only
- Not acoustic mouth-to-ear measurement
- Not measured one-way network latency
- Not transport selection
- No production-readiness claim

## Manual run

See `docs/quality/live-two-peer-runbook.md`. Import endpoint JSON to `.local/live-observation-import/` (gitignored) before pairing.
