# Pi Frontend Split — Workstation vs CT101

**Date:** 2026-07-08  
**Status:** Topology current; **operator frontend superseded 2026-07-10** — see [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md) (`gzmo_cli` canonical, Pi optional).

---

## Summary

**Daemon placement** (unchanged): `gzmo daemon` runs headless on **CT101**; workstation holds dev CLI + operator REPL.

**Operator frontend (2026-07-10):** **`gzmo_cli`** (`gzmo` / `gzmo chat`, `gzmo assemble`, `gzmo memory *`) — not Pi. Pi may still run on the workstation as optional auxiliary.

Earlier note: Pi coding-agent REPL on workstation was intentional when daemon moved to Proxmox; product UI is now gzmo_cli.

---

## Node responsibilities

| Component | CT101 (`.202`) | Workstation (`.184`) |
|-----------|------------------|----------------------|
| `gzmo daemon` (systemd) | Yes — 24/7 background | No (clone for dev CLI only) |
| Pi agent REPL / TUI | **No** | Optional (`pi`, `~/.pi/agent/`) — not product UI |
| **`gzmo chat` (operator)** | No | **Yes** — canonical frontend |
| `vault.db` SoT | Yes (`/opt/gzmo/data/`) | Dev clone only |
| Neo4j / Qdrant / Redis | Yes (Docker) | Remote via LAN |
| Prime `:8000` | Fallback target only | Yes (ornith-35b llama-server) |
| OpenRouter (cloud cognition) | **Primary** for daemon (`active_mode=cloud`) | **Primary** for Pi (`defaultProvider=openrouter`) |
| Local Prime `:8000` | Fallback (`[engine.local]` → workstation) | **Opt-in** (`pi --provider local --model ornith-35b`) |

---

## Dual-cloud cognition (by design)

Both interactive Pi and headless CT101 daemon use **cloud models by default**:

| Layer | Pi (workstation) | gzmo-daemon (CT101) |
|-------|------------------|---------------------|
| Default reasoning | OpenRouter `deepseek/deepseek-v4-flash` | OpenRouter `z-ai/glm-5.2` + `reasoning_effort=xhigh` (`active_mode=cloud`) |
| Local fallback | Ornith-35B `@ localhost:8000` (manual) | Ornith-35B `@ 192.168.31.184:8000` (`[engine.local]`, background failover) |
| Memory / RAG | Sidecar via MCP + `pi-gzmo-memory.sh` | Sidecar colocated (Neo4j/Qdrant/Redis on same LXC) |

Pi does **not** default to local Prime — cloud is the intended operator experience, matching the daemon's cloud-first background routing (`cloud_first_background = true` on CT101).

---

## Engine routing on CT101

```toml
[engine]
active_mode = "cloud"          # Daemon cognition via OpenRouter (by design)

[engine.local]
url = "http://192.168.31.184:8000/v1"   # Workstation Prime — fallback only

[routing]
cloud_first_background = true  # dream/spark/ingest try cloud first, then Prime
```

**Do not** set `active_mode = "local"` on CT101 unless you explicitly want the headless daemon to stop using cloud as its primary engine profile.

See [`CLOUD_MODE_DIAGNOSIS_2026-06-07.md`](./CLOUD_MODE_DIAGNOSIS_2026-06-07.md) for the two-mechanism model (`active_mode` vs `cloud_first_background`).

---

## How Pi connects to the sidecar

Pi on the workstation reaches CT101 services over LAN:

| Pi MCP server | Target |
|---------------|--------|
| `memory` | Neo4j `bolt://192.168.31.202:7687` |
| `gzmo-memory` | Local `gzmo mcp-serve` → reads workstation `gzmo.toml`, vault/Qdrant on CT101 |

Config: `~/.pi/agent/settings.json`

Session distillation: CT101 `[synapse_pull]` polls `data/Synapse/events.jsonl` and runs `gzmo distill` on Pi `session_end` — Pi events must reach the Synapse bus (workstation → CT101 path).

---

## What CT101 is missing (expected gaps)

- No interactive Pi TUI / REPL on the container
- No `/opt/gzmo/.pi/` agent home (legacy `survey_GZMO/.pi/` has stale WORKING_MEMORY only)
- No local Prime on `:8000` (was `localhost:8000` in old config — broken after migration)

These are **not bugs** — Pi frontend relocation is the design.

---

## Ops checklist

```bash
# Pi on workstation
pi   # or however you launch the agent

# Daemon on CT101 (headless)
ssh root@192.168.31.200 "pct exec 101 -- systemctl status gzmo-daemon"

# Verify cloud mode after config change
ssh root@192.168.31.200 "pct exec 101 -- journalctl -u gzmo-daemon --since '2 min ago' | grep 'mode=cloud'"
```
