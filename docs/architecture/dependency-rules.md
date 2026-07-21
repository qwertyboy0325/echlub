# Dependency Rules

```text
echlub-model
    ↑ kernel, collaboration, protocol
             ↑ session, replication
                    ↑ echlub-web, protocol-lab

control-plane → model, protocol, session
```

Core crates must not depend on browser/server/UI frameworks.

Only `echlub-web` uses WASM bindings.

Only `apps/control-plane` uses Axum.
