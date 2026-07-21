---
name: echlub-acceptance-packet
description: Format EchLub bootstrap/work-package acceptance YAML packets.
---

# EchLub Acceptance Packet

Emit compact YAML with:

- work_package_id, result, commits, toolchains
- validation step pass/fail
- audit verdict and blocking findings
- known_limitations, scope_not_implemented
- model accounting fields

Include command excerpts only for failures.

See `docs/quality/acceptance.md` for required checks.
