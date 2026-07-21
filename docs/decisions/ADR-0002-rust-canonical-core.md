# ADR-0002 Rust Canonical Core

## Decision

Canonical semantic logic lives in Rust crates shared by WASM, labs, and future native adapters.

## Consequences

TypeScript remains a presentation shell; domain validation is not duplicated in the browser.
