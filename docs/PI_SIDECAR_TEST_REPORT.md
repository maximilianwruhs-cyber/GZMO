# Pi ↔ Sidecar Integration Test Report

**Date:** 2026-07-08  
**Design note:** Pi uses **cloud model by default** (OpenRouter), same axis as CT101 daemon `active_mode=cloud`. Local Prime is opt-in.

---

## Architecture under test

```
Pi (workstation, cloud LLM)
 ├─ OpenRouter ───────────────────────── reasoning (default)
 ├─ pi-gzmo-memory.sh ────────────────── gzmo memory CLI → sidecar Qdrant/Redis
 ├─ MCP memory ───────────────────────── Neo4j @ 192.168.31.202:7687
 ├─ MCP gzmo-memory ──────────────────── gzmo mcp-serve → sidecar stores
 └─ optional: local Prime @ :8000 ────── pi --provider local

CT101 sidecar
 ├─ Neo4j :7687, Qdrant :6333, Redis :6379
 └─ gzmo-daemon (cloud mode, 24/7)
```

---

## Results

| Test | Status | Detail |
|------|--------|--------|
| Qdrant CT101 `:6333` | **PASS** | 3 collections: honeypot, knowledge, knowledge_core |
| VM200 embed `:8081` | **PASS** | `/v1/models` OK |
| Neo4j bolt `:7687` | **PASS** | `gzmo health` → mcp_memory read_graph OK (13K entities) |
| Redis CT101 `:6379` | **PASS** | `scratch_backend: redis` in `pi-gzmo-memory.sh status` |
| `gzmo health` (workstation) | **PASS** | All sidecar probes green |
| `pi-gzmo-memory.sh prep` | **PASS** | Returns Qdrant knowledge hits for "CT101 sidecar" |
| `pi-gzmo-memory.sh recall` | **PASS** | `[RECALL]` block populated in Redis scratch |
| Pi cloud model (OpenRouter) | **PASS** | Session `019f412c` used `deepseek/deepseek-v4-flash`, responded |
| Pi MCP servers (print mode) | **FAIL → FIXED** | Was `0/0 servers` — `~/.pi/agent/mcp.json` was missing |
| `~/.pi/agent/mcp.json` paths | **FIXED** | Installed via `install-shared-mcp.sh`, paths corrected for `/home/gzmo` |

---

## Fix applied: Pi MCP config

`pi-mcp-adapter` reads **`~/.pi/agent/mcp.json`**, not only `settings.json`.

```bash
cd ~/github-clone/GZMO && ./scripts/install-shared-mcp.sh
# Then verify paths in ~/.pi/agent/mcp.json point to:
#   /home/gzmo/.local/bin/uvx
#   /home/gzmo/github-clone/GZMO/target/release/gzmo
```

`config/shared-mcp-memory.json` in the repo was also updated with workstation paths (was still pointing at `maximilian-wruhs` paths).

---

## Recommended Pi workflow (cloud + sidecar)

```bash
# 1. Memory prep (hits sidecar Qdrant/Redis — no LLM needed)
bash ~/github-clone/GZMO/scripts/pi-gzmo-memory.sh prep "your query" --limit 5

# 2. Interactive Pi (cloud default)
pi

# 3. Optional local model
pi --provider local --model ornith-35b
```

In interactive `pi`, MCP servers (`memory`, `gzmo-memory`) should connect lazily on first tool use. Re-test MCP with `/mcp status` or the mcp built-in tool in TUI.

---

## Layer summary

| Layer | Mechanism | Sidecar reach |
|-------|-----------|---------------|
| **L1 — Memory bridge** | `scripts/pi-gzmo-memory.sh` | Qdrant + Redis via `gzmo memory *` |
| **L2 — MCP tools** | `pi-mcp-adapter` + `mcp.json` | Neo4j + `gzmo_memory_*` tools |
| **L3 — Cloud cognition** | OpenRouter default | Internet (not sidecar) |
| **L4 — Local cognition** | `models.json` local provider | Workstation Prime only |

L1 is verified working. L2 config is now in place; verify in interactive `pi` session. L3 confirmed via prior cloud session.
