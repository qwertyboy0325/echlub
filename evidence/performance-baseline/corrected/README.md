# Corrected synthetic observations

Post-correction valid synthetic runs are stored in per-run directories:

```
corrected/<run-id>/observation.json
corrected/<run-id>/manifest.json
```

Each run must pass both `validate` and `assess-synthetic`.

Historical root-level `synthetic-*.json` artifacts are retained but reclassified as `FAILED_OBSERVATION` — see `docs/quality/performance-baseline-reassessment.md`.
