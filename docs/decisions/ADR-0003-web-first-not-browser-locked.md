# ADR-0003 Web-First Not Browser-Locked

## Decision

Browser is the default client, but core crates remain platform-neutral.

## Consequences

No DOM/WebRTC/WebSocket imports in core crates. Platform effects stay behind adapters.
