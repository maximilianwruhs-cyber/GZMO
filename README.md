# GZMO

**Sovereign, curated long-term memory for coding agents** — honeypot quality gate, supersession chains, local SQLite — exposed as MCP for Cursor and Pi.

Not another cloud notebook. Not Mem0.

## Install (under 10 minutes)

### 1. Install binary + `~/.gzmo` + wire MCP

```bash
curl -fsSL https://raw.githubusercontent.com/maximilianwruhs-cyber/GZMO/main/scripts/install-gzmo.sh | bash
```

Writes SQLite-only `~/.gzmo/` (no LAN) and merges `gzmo-memory` into Cursor / Pi mcp.json.

### 2. Verify on the machine

```bash
# From a clone (or after install):
./scripts/product-stranger-path.sh   # cold product verify checklist
./scripts/mcp-attach-check.sh        # Cursor/Pi point at ~/.gzmo (not CT101)
# Optional feel-it (needs local LLM, e.g. Prime :8000):
./scripts/product-first-fact.sh
```

### 3. Attach in the agent

In Cursor or Pi, call:

1. `gzmo_memory_status` — vault path should be under `~/.gzmo/`  
2. `gzmo_memory_search` — search your local vault  

Full guide: **[docs/PRODUCT_MCP.md](docs/PRODUCT_MCP.md)**.

### Pi package (optional UX)

```bash
pi install npm:gzmo-pi   # or: pi install git:github.com/maximilianwruhs-cyber/gzmo-pi
```

In Pi: `/gzmo setup` → `/reload` → `gzmo_memory_status`.  
Package: [gzmo-pi](https://github.com/maximilianwruhs-cyber/gzmo-pi). Living Redis/Neo4j/Qdrant is **not** required for laptop product.

### From source

```bash
cargo build --release -p gzmo-cli
GZMO_BIN=./target/release/gzmo ./scripts/install-gzmo.sh
```

## What you get

| | |
|---|---|
| **Local vault** | `~/.gzmo/data/vault.db` — facts stay on disk |
| **Honeypot curation** | Quality-gated recall, not a raw dump |
| **Supersession chains** | `gzmo_memory_chain` shows how facts were replaced |
| **MCP stdio** | Works with Cursor and Pi |
| **FTS-first** | Offline by default; optional embeddings later |

## MCP tools

| Tool | Purpose |
|------|---------|
| `gzmo_memory_status` | Vault path, fact counts, session |
| `gzmo_memory_search` | Search honeypot / vault |
| `gzmo_memory_turn_start` | Clear session scratch |
| `gzmo_memory_recall_pull` | Pull scratch block |
| `gzmo_memory_chain` | Provenance / supersession chain |
| `gzmo_memory_profile` | Cached operator profile |
| `gzmo_wiki_search` | Wiki layer (if enabled) |

Operator-only probes (`gzmo_ops_health`, `gzmo_discovery_status`) require `GZMO_OPS_MCP=1`.

## Non-goals (v1)

- Multi-host living topologies and discovery / mentor pedagogy as install steps  
- Cloud-hosted memory SaaS — use [Mem0 MCP](https://docs.mem0.ai/) if you want that  
- Overnight `gzmo serve`, Redis, Qdrant, or Neo4j required for first attach  
- SEIP / Foundry-class platforms inside this product surface  

If a change does not help a stranger get `gzmo_memory_search` working in under 10 minutes, it is not product.

## Optional next steps

After MCP works you can enable `[embeddings]` against any OpenAI-compatible local URL. Redis/Qdrant compose and overnight metabolism are **advanced** — see [docs/PRODUCT_MCP.md](docs/PRODUCT_MCP.md#advanced-phase-2--deferred).

## Docs

| Doc | Audience |
|-----|----------|
| [docs/PRODUCT_MCP.md](docs/PRODUCT_MCP.md) | Product install + verify |
| [docs/README.md](docs/README.md) | Operator / architecture index |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Dev setup |

## License

[MIT](LICENSE)
