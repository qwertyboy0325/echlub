# Dependency Rules

```text
echlub-model
    ↑ kernel, collaboration, protocol
             ↑ session, replication
                    ↑ echlub-web, protocol-lab

control-plane → model, protocol, session

echlub-jam
    ↑ echlub-musician-sim, jam-relay
             ↑ jam-lab
```

`echlub-jam` and `echlub-musician-sim` must stay platform-neutral (no sockets,
threads, audio devices, or browser APIs) and must build for
`wasm32-unknown-unknown`.

Core crates must not depend on browser/server/UI frameworks.

Only `echlub-web` uses WASM bindings.

Only `apps/control-plane` uses Axum.
