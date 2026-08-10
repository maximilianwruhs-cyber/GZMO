# Shared MCP client inventory (workstation)

**Date:** 2026-08-10  
**Ticket:** [#154 Shared MCP client inventory](https://github.com/maximilianwruhs-cyber/GZMO/issues/154) (map [#151](https://github.com/maximilianwruhs-cyber/GZMO/issues/151))  
**Status:** Research note for the shared-MCP chapter of the living-attach config spec  
**Host:** this workstation (`HOME=/home/gzmo`)  
**Method:** read `scripts/install-shared-mcp.sh` target constants; probe each path on disk; list `mcpServers` keys (especially `gzmo-living` vs `gzmo-memory`)

---

## 1. Script targets (primary source)

`scripts/install-shared-mcp.sh` defines three **merge write** homes and one **product restore** fragment:

| Variable | Path | Role | Citation |
|----------|------|------|----------|
| `CURSOR_MCP` | `~/.cursor/mcp.json` | merge target | L9 |
| `PI_MCP` | `~/.pi/agent/mcp.json` | merge target | L10 |
| `GLOBAL_MCP` | `~/.config/mcp/mcp.json` | merge target | L11 |
| `PRODUCT_FRAG` | `~/.gzmo/mcp.json` | restore source for product `gzmo-memory` if living rename emptied the name (L100–L110); **not** a merge destination | L13 |

Merge calls: L115–L117 (`Cursor MCP`, `Pi MCP`, `Global shared MCP`).

Fragment merged in: `config/shared-mcp-memory.json` (`FRAG`, L8) — ships `memory` + `gzmo-living` only (no `gzmo-memory` in the fragment).

---

## 2. On-disk inventory (2026-08-10)

Absolute paths expanded under `/home/gzmo`.

| Home | Exists? | Server names present | `gzmo-living` / `gzmo-memory` verdict |
|------|---------|----------------------|----------------------------------------|
| `/home/gzmo/.cursor/mcp.json` | **yes** | `gzmo-living`, `gzmo-memory`, `memory` | **both** |
| `/home/gzmo/.pi/agent/mcp.json` | **yes** | `gzmo-living`, `memory` | **`gzmo-living` only** (no `gzmo-memory`) |
| `/home/gzmo/.config/mcp/mcp.json` | **yes** | `gzmo-living`, `gzmo-memory`, `memory` | **both** |
| `/home/gzmo/.gzmo/mcp.json` (`PRODUCT_FRAG`) | **no** | — | N/A (missing restore source) |

### 2.1 Cursor — both

Path: `/home/gzmo/.cursor/mcp.json`

- `gzmo-living` → `/home/gzmo/github-clone/GZMO/scripts/pi-gzmo-mcp-serve.sh` (args `[]`)
- `gzmo-memory` → `/home/gzmo/.local/bin/gzmo` args `["mcp-serve"]` (lite / product-shaped)
- `memory` → Neo4j MCP via `uvx` from `/home/gzmo/github-clone/mcp-neo4j-memory-gzmo`

### 2.2 Pi — living only

Path: `/home/gzmo/.pi/agent/mcp.json`

- `gzmo-living` → same `pi-gzmo-mcp-serve.sh` wrapper
- `memory` → same Neo4j MCP
- **No** `gzmo-memory` entry

### 2.3 Global shared — both

Path: `/home/gzmo/.config/mcp/mcp.json`

- Same trio as Cursor: `gzmo-living`, `gzmo-memory`, `memory` with the same command shapes.

### 2.4 Product fragment — absent

`~/.gzmo/mcp.json` does not exist on this workstation. The installer can still leave `gzmo-memory` on Cursor/global if it was already present; it cannot restore product MCP from `PRODUCT_FRAG` until that file exists (script L100–L110).

---

## 3. Out-of-script MCP-related JSON (not covered by `install-shared-mcp.sh`)

Discovered under common client trees; **not** written by L115–L117:

| Path | Notes |
|------|--------|
| `/home/gzmo/.pi/agent/mcp-cache.json` | Pi cache; `servers` keys include `gzmo-living`, `gzmo-memory`, `memory` — **stale vs live Pi mcp.json** (live file has no `gzmo-memory`). Do not treat as config home. |
| `/home/gzmo/.cursor/projects/home-gzmo/mcp-approvals.json` | Cursor project approvals metadata, not an `mcpServers` merge home. |
| `/home/gzmo/github-clone/GZMO/data-next/living-appliance-home/mcp.json` | In-repo / appliance sample tree; only `gzmo-memory` in `mcpServers`. Not a workstation client home targeted by the shared installer. |
| Repo fragments `config/shared-mcp-memory.json`, `config/product-mcp-memory.json` | Source fragments for installers, not client homes. |

No additional `mcp.json` under `~/.openclaw` or `~/.claude` turned up in a shallow `*mcp*.json` walk for this inventory.

---

## 4. Spec takeaways (shared-MCP chapter)

1. **All three merge targets exist** on this ops workstation; none are missing.
2. **Living label is present everywhere the script writes** (`gzmo-living` on Cursor, Pi, and global) — aligns with GREEN expectation “MCP name `gzmo-living` in each configured client.”
3. **Product label is uneven:** Cursor + global still carry `gzmo-memory` (lite `gzmo mcp-serve`); Pi does not. Spec should say whether coexistence is allowed, required, or refused on ops boxes.
4. **`PRODUCT_FRAG` is missing** here — document restore-from-`~/.gzmo/mcp.json` as best-effort, and that absence does not block living merge.
5. **Do not confuse caches/approvals/samples** with installer homes when writing refuse conditions or proof commands.

---

## 5. One-line gist

All three `install-shared-mcp.sh` homes exist: Cursor + global have **both** `gzmo-living` and `gzmo-memory`; Pi has **living only**; `~/.gzmo/mcp.json` product fragment is absent.
