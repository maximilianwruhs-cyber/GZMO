# GZMO product: local memory MCP for Cursor / Pi

Install a **sovereign, curated long-term memory** for coding agents — honeypot quality gate, supersession chains, and local SQLite — exposed as MCP. Not a cloud notebook; not Mem0.

## 5-minute install

### 1. Install binary + init + MCP

```bash
# Linux x86_64 — preferred
curl -fsSL https://raw.githubusercontent.com/maximilianwruhs-cyber/GZMO/main/scripts/install-gzmo.sh | bash

# Private repo: export GH_TOKEN=ghp_... first (repo scope)
# Or from a clone with a local release build:
#   cargo build --release -p gzmo-cli && ./scripts/install-gzmo.sh
```

This:

1. Installs `gzmo` to `~/.local/bin`
2. Runs `gzmo init` → `~/.gzmo/` (SQLite vault, embeddings off, Redis/Qdrant off)
3. Merges `gzmo-memory` into Cursor / Pi / global `mcp.json`

No LAN hosts. No remote living stack required.

Manual fragment (if you skip the installer merge):

```json
{
  "mcpServers": {
    "gzmo-memory": {
      "command": "/home/you/.local/bin/gzmo",
      "args": ["mcp-serve"],
      "env": {
        "GZMO_CONFIG": "/home/you/.gzmo/gzmo.toml",
        "GZMO_ALLOW_LAB_VAULT": "1",
        "GZMO_PRODUCT": "1"
      }
    }
  }
}
```

### 2. Verify

In Cursor/Pi, call:

- `gzmo_memory_status` — vault path + fact counts
- `gzmo_memory_search` — FTS over the local vault (empty until you ingest/promote)

CLI smoke:

```bash
./scripts/verify-product-mcp.sh
# or:
export GZMO_CONFIG="$HOME/.gzmo/gzmo.toml" GZMO_ALLOW_LAB_VAULT=1 GZMO_PRODUCT=1
gzmo memory status --json
```

### Releases

Tagged builds (`v*`) publish `gzmo-x86_64-unknown-linux-gnu.tar.gz` via GitHub Actions (`.github/workflows/release.yml`).

## Product tools (default)

| Tool | Purpose |
|------|---------|
| `gzmo_memory_turn_start` | Clear session scratch |
| `gzmo_memory_search` | Search honeypot/vault |
| `gzmo_memory_status` | Vault path + counts |
| `gzmo_memory_recall_pull` | Session scratch block |
| `gzmo_memory_chain` | Supersession / provenance chain |
| `gzmo_memory_profile` | Cached operator profile |
| `gzmo_wiki_search` | Wiki layer (if enabled in config) |

## Ops / discovery tools (gated)

`gzmo_ops_health` and `gzmo_discovery_status` stay available for operators but are **gated**:

```bash
GZMO_OPS_MCP=1
```

Without that env var they return a clear error and are not part of the product story.

## Optional embeddings

Default product config uses **FTS-only** (offline). To enable vectors later, set in `~/.gzmo/gzmo.toml`:

```toml
[embeddings]
enabled = true
url = "http://127.0.0.1:8002/v1"   # any OpenAI-compatible embeddings API
model = "your-embedding-model"
```

Still no LAN topology required.

## Non-goals (v1 product)

- Multi-host living topology and operator discovery timers
- Overnight dream/spark/distill as a required install step
- SEIP / Foundry-class ingestion platforms (separate research)
- Competing with Mem0 cloud “connect in minutes” — if you need that, use Mem0 MCP

## Advanced (Phase 2 — deferred)

Documented only; not required for MCP attach:

- `docker compose` for Redis + Qdrant when you want vector recall sidecars
- `gzmo serve` / daemon overnight metabolism as an **optional** advanced path

See operator docs under `docs/` for the private living stack. Those paths are not the outsider install funnel.
