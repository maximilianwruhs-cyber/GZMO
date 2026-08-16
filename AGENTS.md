# GZMO — Agent Guide

Sovereign Rust agent: honeypot memory pipeline + local LLM. Read `MACHINE.md` first.

**USP:** full living Keep on one airgapped box — [docs/ADR-0004-airgap-living-usp.md](docs/ADR-0004-airgap-living-usp.md) · one product [docs/ADR-0007-one-product-living.md](docs/ADR-0007-one-product-living.md). Quality: `bash scripts/keep-quality-gate.sh`. No lite SKU — clients attach to the living writer. Active Unpark nutrients: [docs/BRAIN_FEED.md](docs/BRAIN_FEED.md) (`bash scripts/brain-feed-check.sh`). What to build next: [docs/OPPORTUNITY_DISCOVERY.md](docs/OPPORTUNITY_DISCOVERY.md) (`bash scripts/opportunity-discovery-check.sh`).

## Repo layout

| Path | Role |
|------|------|
| `gzmo-core/` | All business logic — config, gateway, memory, ingest, dream, spark, MCP |
| `gzmo-cli/` | Thin binary: `main.rs` + `*_cmd.rs` + optional TUI |
| `gzmo-chaos/` | Lorenz attractor engine (separate crate) |
| `gzmo.toml` | **Local** operator config (gitignored — copy from `gzmo.toml.example`) |
| `.env` | **Local** secrets (gitignored — copy from `.env.template`) |
| `memory/` | Episodic logs (runtime, gitignored) |
| `data/vault.db` | SQLite vault (runtime, gitignored) |
| `wiki/` | Git-tracked markdown wiki layer (Obsidian-browsable) — see `WIKI.md` |
| `WIKI.md` | Wiki schema + conventions (how GZMO maintains `wiki/`) |
| `scripts/` | Production ops — prefer these over ad-hoc commands |
| `docs/` | Canonical docs — see `docs/README.md` |

## Conventions

- **Minimize scope** — focused diffs, match existing Rust style in `gzmo-core`.
- **Secrets** — never in committed files; `.env` + `apply_mcp_env_overrides` in `config.rs`.
- **Two skill systems** — Rust skills in `gzmo-core/src/skills/`, shell skills in `skills/` + `scripts/skill_*.sh`.
- **Engines** — Prime at `:8000`, embeddings VM200 `:8081`, Qdrant LXC101 `:6333`.
- **Pipeline** — extract → verify → promote → vault → honeypot (see `MACHINE.md`).

## Verify changes

```bash
cargo test
cargo clippy --all-targets
./scripts/verify-production.sh    # needs live infra
```

## Do not touch without reason

- `data/lore.toml` — static chaos lore seed (tracked)
- `SOUL.md` — agent persona (tracked)
- `docs/archive/` — local session notes (gitignored)
