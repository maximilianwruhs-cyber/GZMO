# Zero-Bloat Reviews

Required before merging changes that add:

- A new entry in `[workspace.dependencies]` or a crate `Cargo.toml` dependency
- A new network client or external HTTP integration
- A new subprocess tool dependency

## Process

1. Copy `_TEMPLATE.md` to `YYYY-MM-DD-<short-name>.md`
2. Fill all sections including Obolus fields
3. Link the review in your PR description

## Baseline

See [BASELINE-2026-06.md](BASELINE-2026-06.md) for the reference dep list and binary size.
