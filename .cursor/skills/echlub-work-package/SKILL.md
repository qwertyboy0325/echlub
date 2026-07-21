---
name: echlub-work-package
description: Execute bounded EchLub work packages with explicit scope and checks.
---

# EchLub Work Package Skill

1. Read the active package under `docs/work-packages/`.
2. Confirm file_scope read/write boundaries.
3. Implement only authorized scope.
4. Run required checks from the package (usually `pnpm verify`).
5. Record model accounting in the completion packet.

Canonical architecture lives in `docs/architecture/` — do not duplicate it here.
