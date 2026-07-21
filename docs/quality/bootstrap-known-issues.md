# Bootstrap Known Issues

## Medium

- `corepack enable` may fail without permission to symlink global binaries; invoke pnpm via `corepack pnpm` instead.
- Web unit tests mock WASM load failure to enforce no TypeScript domain fallback; browser WASM integration is validated via `wasm-pack` tests and production build.

## Low

- `echlub-protocol-lab` emits Clippy warnings for unused imports (non-blocking).

No critical or high severity defects recorded after foundation audit correction round.
