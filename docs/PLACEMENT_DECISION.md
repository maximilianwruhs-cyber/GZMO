# GZMO Daemon Placement Decision

**Date:** 2026-07-08  
**Status:** Accepted  
**Decision:** **Keep Rust `gzmo-daemon` on CT101** (Proxmox sidecar); workstation remains cognition + dev node.

---

## Context

The canonical docs ([`INFRASTRUCTURE_MAP.md`](./INFRASTRUCTURE_MAP.md)) describe the workstation as the source-of-truth node with `vault.db` local. Live production (since sidecar migration) runs:

- **Daemon + vault.db** on CT101 (`/opt/gzmo/`)
- **Prime LLM** on workstation (`192.168.31.184:8000`)
- **Retrieval** on VM200 (`:8081` embed router)
- **GraphRAG stores** on CT101 Docker (Neo4j, Qdrant, Redis)

The Jul 7 infrastructure report proposed consolidating everything back to the workstation. After implementing the plan (Prime online, CT101 pointed at workstation Prime), we evaluated both options.

---

## Options considered

### A. Keep daemon on CT101 (chosen)

| Pros | Cons |
|------|------|
| 24/7 autonomous pipelines (dream 01:00, spark, pulse) without workstation uptime | Split-brain: vault on CT101, Prime on workstation |
| Colocated with Neo4j/Qdrant/Redis — low-latency MCP + vector sync | No Pi REPL on CT101 — interactive frontend on workstation |
| Workstation GPU free for Pi/Cursor/herdr | `active_mode=cloud` + OpenRouter dependency for daemon cognition |
| Proven stable: 16K+ journal lines, 689 MB vault, active since Jun 3 | SSH `pct exec` ops slightly more complex |

### B. Consolidate to workstation

| Pros | Cons |
|------|------|
| Matches canonical INFRASTRUCTURE_MAP placement | Workstation must stay on for nightly dream/spark |
| Single `gzmo.toml`, single `vault.db` path | 689 MB vault migration + downtime risk |
| Simpler mental model for Pi agent | Competes with Prime for RAM/CPU on dev machine |
| Local systemd `gzmo-daemon.service` already templated | Loses sidecar isolation (Docker DBs still on CT101) |

### C. Hybrid (current architecture)

- **CT101:** headless daemon (`active_mode=cloud`), vault.db, Docker data plane
- **Workstation:** Pi frontend, Prime, herdr, dev clones
- **VM200:** embeddings/rerank

This is what is running today and what we commit to.

---

## Rationale

1. **Autonomy requirement:** DreamEngine, SparkEngine, PulseLoop, and Qdrant nightly sync need continuous uptime. The workstation sleeps/reboots; CT101 does not.

2. **Cloud cognition on daemon:** CT101 `active_mode=cloud` is **by design** — headless daemon uses OpenRouter for scheduled cognition. `cloud_first_background=true` routes dream/spark/ingest cloud-first with Prime fallback at `http://192.168.31.184:8000/v1`.

3. **Pi frontend on workstation:** Interactive Pi REPL lives at `~/.pi/agent/` on the workstation, not on CT101. See [`PI_FRONTEND_SPLIT.md`](./PI_FRONTEND_SPLIT.md).

4. **Data locality:** `vault.db`, Neo4j MCP (stdio on CT101), and Qdrant sync all run on the same LXC.

5. **TypeScript daemon stays separate:** Per [`frankenstein/gzmo/README.md`](../../frankenstein/gzmo/README.md), the Bun `gzmo-daemon` (gzmo_tinyFolder) is the v1 inbox spine — never run alongside Rust GZMO on the same vault.

---

## Workstation role

| Component | Runs on |
|-----------|---------|
| **Pi agent frontend** (REPL/TUI) | Workstation `~/.pi/agent/` |
| Prime `llama-server :8000` | Workstation (Pi local fallback + daemon Prime fallback) |
| Pi agent / Cursor / herdr | Workstation |
| Rust `gzmo` CLI for dev/testing | Workstation clone (`github-clone/GZMO`) |
| Rust `gzmo daemon` production (headless) | **CT101** — `active_mode=cloud` |

Optional: enable `gzmo-prime.service` on workstation for boot persistence of Prime.

---

## Ops implications

- **Health checks:** `pct exec 101 -- gzmo health` + workstation `after-boot-verify.sh`
- **Config changes:** Edit `/opt/gzmo/gzmo.toml` on CT101; restart `gzmo-daemon`
- **Prime changes:** Workstation model swap does not require CT101 restart if model path in `gzmo.toml` is updated
- **Future consolidation:** Revisit only if CT101 RAM/disk becomes constrained or workstation gets 24/7 UPS + always-on policy

---

## Related docs

- [`SIDECAR_SSH_STATUS.md`](./SIDECAR_SSH_STATUS.md) — SSH access matrix
- [`SIDECAR_HEALTH_REPORT.md`](./SIDECAR_HEALTH_REPORT.md) — live health snapshot
- [`PI_FRONTEND_SPLIT.md`](./PI_FRONTEND_SPLIT.md) — Pi on workstation, daemon headless on CT101
