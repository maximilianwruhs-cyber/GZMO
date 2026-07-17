# GZMO product: local memory MCP for Cursor / Pi

Install a **sovereign, curated long-term memory** for coding agents — honeypot quality gate, supersession chains, and local SQLite — exposed as MCP. Not a cloud notebook; not Mem0.

## 5-minute install

### 1. Build or download the binary

```bash
cargo build --release -p gzmo-cli
# binary: target/release/gzmo
# optional: copy to ~/.local/bin/gzmo
```

GitHub Release assets (when published): download `gzmo`, `chmod +x`, put it on your `PATH`.

### 2. Initialize a laptop vault

```bash
gzmo init
```

Writes under `~/.gzmo/`:

- `gzmo.toml` — SQLite vault, embeddings off (FTS-only), Redis/Qdrant/Neo4j off
- `data/vault.db` — empty vault (lab attach allowed)
- `mcp.json` — Cursor/Pi fragment for `gzmo-memory`

No LAN hosts. No remote living stack required.

### 3. Attach MCP

```bash
./scripts/install-product-mcp.sh
# or paste ~/.gzmo/mcp.json into ~/.cursor/mcp.json / ~/.pi/agent/mcp.json
```

Fragment shape:

```json
{
  "mcpServers": {
    "gzmo-memory": {
      "command": "/path/to/gzmo",
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

### 4. Verify

In Cursor/Pi, call:

- `gzmo_memory_status` — vault path + fact counts
- `gzmo_memory_search` — FTS over the local vault (empty until you ingest/promote)

CLI smoke (no IDE):

```bash
export GZMO_CONFIG="$HOME/.gzmo/gzmo.toml"
export GZMO_ALLOW_LAB_VAULT=1
export GZMO_PRODUCT=1
gzmo mcp-serve   # stdio; use with an MCP client
```

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
