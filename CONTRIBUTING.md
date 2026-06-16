# Contributing to GZMO

## First-time setup

```bash
git clone <repo-url> && cd survey_GZMO

# Local config (not committed)
cp gzmo.toml.example gzmo.toml
cp .env.template .env
# Edit .env — set NEO4J_PASSWORD and optional API keys

# Runtime directories (created automatically by `gzmo init`, or manually)
mkdir -p memory data logs

cargo build --release
cargo test
```

## What not to commit

- `.env`, `gzmo.toml` (machine-specific config)
- `data/vault.db`, `memory/*.md`, `data/sessions/`, `logs/`
- `docs/archive/` session reports
- Models, binaries, `.pi/` agent state

See `.gitignore` for the full list.

## Code layout

```
gzmo-core/   # All agent logic (config, memory, tools, daemon)
gzmo-cli/    # Binary entrypoint + TUI
gzmo-chaos/  # Lorenz chaos engine crate
scripts/     # Ops, systemd, eval harnesses
skills/      # Shell-based slash skills (Rust skills live in gzmo-core)
docs/        # Canonical documentation (see docs/README.md)
```

## Before opening a PR

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
./scripts/sovereignty-verify.sh   # ARCH-DIR + Obolus (no live infra)
./scripts/verify-production.sh   # if infra is up
```

New `[workspace.dependencies]` entries require a file in `docs/zero-bloat-reviews/`.

## Secrets

Never put passwords or API keys in `gzmo.toml` or committed files. Use `.env` — GZMO overlays MCP and API credentials from there at startup.
