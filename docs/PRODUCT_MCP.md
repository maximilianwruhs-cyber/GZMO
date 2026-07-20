# GZMO lite: local memory MCP for Cursor / Pi (bootstrap)

**Profile:** lite bootstrap — not the USP. Full product dream is **airgap living** on one box: [AIRGAP_LIVING.md](./AIRGAP_LIVING.md) · [ADR-0004-airgap-living-usp.md](./ADR-0004-airgap-living-usp.md).

Install a **sovereign, curated long-term memory** for coding agents — honeypot quality gate, supersession chains, and local SQLite — exposed as **stdio MCP**. Not a cloud notebook; not Mem0; not a public webserver.

## 5-minute install

### 0. Pi users (recommended)

Install the Pi package ([gzmo-pi](https://github.com/maximilianwruhs-cyber/gzmo-pi)) which bundles `pi-mcp-adapter` and wires product MCP:

```bash
# Binary + ~/.gzmo (if not already)
curl -fsSL https://raw.githubusercontent.com/maximilianwruhs-cyber/GZMO/main/scripts/install-gzmo.sh | bash

pi install npm:gzmo-pi
```

In Pi: `/gzmo setup` → `/reload` → `gzmo_memory_status`.  
For Redis / Neo4j / Qdrant living topology see [gzmo-pi README — Living stack](https://github.com/maximilianwruhs-cyber/gzmo-pi#living-stack-redis--neo4j--qdrant) and operator docs in this repo.

### 1. Install binary + init + MCP

```bash
# Linux x86_64 — preferred
curl -fsSL https://raw.githubusercontent.com/maximilianwruhs-cyber/GZMO/main/scripts/install-gzmo.sh | bash

# Or from a clone:
#   cargo build --release -p gzmo-cli && ./scripts/install-gzmo.sh
```

This:

1. Installs `gzmo` to `~/.local/bin`
2. Runs `gzmo init` → `~/.gzmo/` (SQLite vault, embeddings off, Redis/Qdrant off)
3. Merges `gzmo-memory` into Cursor / Pi / global `mcp.json`

`gzmo init` scans localhost and **prefers Prime / llama.cpp on `:8000`** when up; otherwise falls back to LM Studio `:1234`. No LAN hosts. No remote living stack required.

Refresh an existing home without wiping the vault:

```bash
gzmo init --refresh-engine
```

Lite gate: `bash scripts/product-readiness-gate.sh` → [PRODUCT_PRODUCTION_READINESS.md](PRODUCT_PRODUCTION_READINESS.md).  
Living USP gate: `bash scripts/keep-quality-gate.sh` → [KEEP_QUALITY.md](KEEP_QUALITY.md).  
Local attach contract: [MCP_LOCAL_ATTACH.md](MCP_LOCAL_ATTACH.md).

Hero path (sidecars + overnight on this machine):

```bash
bash scripts/install-living-airgap.sh
```

Manual lite fragment (if you skip the installer merge):

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

CLI smoke (stranger checklist + attach gate):

```bash
bash scripts/product-stranger-path.sh
# → data-next/product-stranger/latest.md  (or ./scripts/verify-product-mcp.sh alone)

bash scripts/mcp-attach-check.sh
# → data-next/mcp-attach/latest.md
# If Cursor still points at a temp-bench binary:
#   MCP_ATTACH_FIX=1 bash scripts/mcp-attach-check.sh

# or manually:
export GZMO_CONFIG="$HOME/.gzmo/gzmo.toml" GZMO_ALLOW_LAB_VAULT=1 GZMO_PRODUCT=1
gzmo memory status --json
# Third surface alias (same as mcp-serve):
gzmo memory mcp
```

**Attach rule:** `GZMO_CONFIG` must be `~/.gzmo/gzmo.toml` with `GZMO_PRODUCT=1`. Never point the laptop product MCP at CT101 `/opt/gzmo` or workstation `data-next/`.

### 3. Feel it (optional first fact)

Needs a live OpenAI-compatible engine (Prime `:8000` by default). Uses a **sibling** overlay `~/.gzmo/gzmo-product-engine.toml` — does not permanently rewrite product `gzmo.toml`.

```bash
bash scripts/product-first-fact.sh
# → data-next/product-first-fact/latest.md
# PRODUCT_ENGINE_URL=http://127.0.0.1:8000/v1  # override if needed

bash scripts/product-hello-memory.sh
# → data-next/product-hello/ — attach + first-fact combined stranger feel
```

**Appliance note:** product install is the Memory MCP appliance (map m2). Living
workstation vault (`data-next/`) is separate — use `GZMO_CONFIG` pointing at
`~/.gzmo/gzmo.toml` for the portable product, not the nightburst living root.

### Releases

Tagged builds (`v*`) publish `gzmo-x86_64-unknown-linux-gnu.tar.gz` via GitHub Actions (`.github/workflows/release.yml`).  
Keep the published tag at tip after Keep features land — `bash scripts/release-freshness-check.sh` (also soft-checked by `product-readiness-gate.sh`). Stranger `curl|bash` installs GitHub **latest**, not your local tip binary.

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

## Advanced / living stack (optional)

Not required for product MCP attach. Operator hosts that want the full topology:

| Service | Role |
|---------|------|
| SQLite | Vault + FTS (always) |
| Redis | Hot session / cache |
| Neo4j / living vault | Operator only — `gzmo-living` via `scripts/install-shared-mcp.sh` (never product `gzmo-memory`) |
| Qdrant | Vectors when embeddings are enabled |

See [gzmo-pi Living stack](https://github.com/maximilianwruhs-cyber/gzmo-pi#living-stack-redis--neo4j--qdrant), `docs/CT101_DEPLOY.md`, and `docs/PI_GZMO_MEMORY_INTEGRATION.md`. Laptop product install stays SQLite-only.
