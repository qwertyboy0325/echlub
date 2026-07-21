# EchLub Agent Guide

Cross-agent entry for the EchLub rewrite monorepo.

## Product

Web-first remote improvisational music system with a Rust canonical composition core and performance baseline lab. Foundation package implements semantic document operations; performance baseline implements exploratory WebRTC instrumentation.

## Proof of behavior

Tests and `scripts/verify.py` are authoritative over documentation summaries.

## Required reading order

1. `README.md`
2. `docs/README.md`
3. `docs/product/vision.md`
4. `docs/architecture/overview.md`
5. Active work package under `docs/work-packages/`
6. Applicable `.cursor/rules/`
7. Applicable `.cursor/skills/`
8. `docs/reference/legacy-archaeology.md` only when legacy context is needed

## Reference repositories

`.reference/**` is read-only, gitignored, and must never become a build dependency.

## Scope boundaries

- No latency claim without evidence
- No general CRDT claim
- No final transport selection
- API-backed models forbidden unless owner explicitly authorizes

## Model budget

See `docs/ai/model-budget.md`. Review date: 2026-08-11.

## Git authorization

Local commits only within authorized work packages. No remote, push, or history rewrite unless owner requests.
