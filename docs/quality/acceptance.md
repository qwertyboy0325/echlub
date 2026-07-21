# Foundation Acceptance

## Required checks (`pnpm verify`)

1. Reference repositories clean and SHA-locked
2. `cargo fmt --check`
3. `cargo clippy -D warnings`
4. `cargo test --workspace`
5. `wasm-pack build` + `wasm-pack test --node`
6. Web typecheck, test, build
7. `protocol-lab` self-check against `test-vectors/foundation-v1.json`
8. Control-plane endpoint tests

## Explicit non-claims

No production readiness, latency improvement, CRDT completeness, or transport selection.
