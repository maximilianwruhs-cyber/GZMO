# GZMO — Sovereign curated memory for coding agents

> **Local SQLite · Honeypot quality gate · MCP for Cursor & Pi**

Install once, point Cursor or Pi at `gzmo mcp-serve`, and get **sovereign long-term memory** with supersession chains — not another cloud notebook, not a Mem0 clone.

```bash
cargo build --release -p gzmo-cli
./target/release/gzmo init          # writes ~/.gzmo/ (SQLite-only, no LAN)
./scripts/install-product-mcp.sh    # merges MCP into Cursor / Pi
# Then call gzmo_memory_status + gzmo_memory_search in your agent
```

Full walkthrough: **[docs/PRODUCT_MCP.md](docs/PRODUCT_MCP.md)**.

---

## What you get

| Capability | Why it matters |
|------------|----------------|
| Local vault (`~/.gzmo/data/vault.db`) | Your facts stay on disk |
| Honeypot + promote path | Curated recall, not raw dump search |
| Supersession chains (`gzmo_memory_chain`) | Provenance when facts are replaced |
| MCP stdio server | Works with Cursor and Pi out of the box |
| FTS-first defaults | Offline first run; optional embeddings later |

**Unique wedge:** *Sovereign, curated long-term memory for coding agents—honeypot quality gate, supersession chains, and local SQLite—exposed as MCP.*

---

## Non-goals (v1 product)

GZMO’s public product surface is **MCP memory**. These are explicitly out of scope for the first download:

- Multi-host living topologies and operator-only discovery / mentor pedagogy
- Cloud-hosted memory SaaS (use [Mem0 MCP](https://docs.mem0.ai/) if you want that)
- SEIP / Foundry-class platforms (NiFi, Iceberg, ontology graphs) — keep that research separate
- Requiring overnight `gzmo serve`, Redis, Qdrant, or Neo4j for first attach

If a change does not help a stranger get `gzmo_memory_search` working in under 10 minutes, it is living-stack or research — not product.

---

## MCP tools (product)

- `gzmo_memory_status` / `gzmo_memory_search` / `gzmo_memory_turn_start`
- `gzmo_memory_recall_pull` / `gzmo_memory_chain` / `gzmo_memory_profile`
- `gzmo_wiki_search` (when `[wiki]` is enabled)

Operator probes (`gzmo_ops_health`, `gzmo_discovery_status`) require `GZMO_OPS_MCP=1`.

---

## Build from source

```bash
# Prerequisites: Rust stable, Linux
cargo build --release -p gzmo-cli
cp target/release/gzmo ~/.local/bin/gzmo   # optional
gzmo init
```

Configuration lives at `~/.gzmo/gzmo.toml` (or set `GZMO_CONFIG`). Sidecars stay **off** until you opt in.

---

## Optional / advanced

After MCP works:

- Enable `[embeddings]` against any OpenAI-compatible local URL for vector recall
- Later: Redis/Qdrant via compose (see [docs/PRODUCT_MCP.md](docs/PRODUCT_MCP.md#advanced-phase-2--deferred))
- Overnight metabolism (`gzmo serve`) is **advanced**, not part of the install funnel

---

## Repository layout

```
GZMO/
├── gzmo-core/       # Vault, honeypot, MCP server, platform memory
├── gzmo-cli/        # `gzmo` binary (init, mcp-serve, …)
├── config/          # MCP fragments (product + operator)
├── scripts/         # install-product-mcp.sh and ops helpers
├── docs/            # Product + operator documentation
└── docs/PRODUCT_MCP.md
```

---

## Operator & architecture docs

Internal living-stack and platform docs stay under [`docs/`](docs/README.md) (roadmap, memory architecture, production readiness). They are **not** the outsider install path.

Contributing / verify: [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

See repository license file.
