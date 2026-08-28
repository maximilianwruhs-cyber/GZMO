# Pi × GZMO living stack (recovered architecture)

**Status:** Recovered 2026-07-19 from docs, live `~/.pi/agent/`, CT101 handoff, and branch `feat/context-compress-headroom`  
**Problem this solves:** The “supreme” Pi wiring was never written as one durable runbook; custom attach broke on every official Pi upgrade.  
**Related:** [PI_GZMO_MEMORY_INTEGRATION.md](PI_GZMO_MEMORY_INTEGRATION.md) · [PI_UPGRADE_RUNBOOK.md](PI_UPGRADE_RUNBOOK.md) · [PI_PACKAGE_ALLOWLIST.md](PI_PACKAGE_ALLOWLIST.md) · [HEADROOM_CCR.md](../HEADROOM_CCR.md) · [OPERATOR_FRONTEND_DECISION.md](../OPERATOR_FRONTEND_DECISION.md)

---

## North star (do not confuse these)

| Mode | Vault | Redis / Qdrant / Neo4j | Pi attach |
|------|-------|------------------------|-----------|
| **Living** | CT101 `/opt/gzmo/` (~60k facts) | **On** (CT101 sidecars) | MCP server **`gzmo-living`** via `install-shared-mcp.sh` |
| **Product** | Laptop `~/.gzmo/` | **Off** | MCP server **`gzmo-memory`** via `install-product-mcp.sh` |

Pi is an **optional auxiliary frontend**. Canonical operator UI is `gzmo` / `gzmo chat` ([OPERATOR_FRONTEND_DECISION.md](../OPERATOR_FRONTEND_DECISION.md)). Pi must **not** invent a parallel Redis client — it talks memory only through GZMO MCP / `pi-gzmo-memory.sh`.

---

## Topology (what “supreme Pi” actually was)

```text
 Pi (workstation and/or CT101)
   packages: pi-mcp-adapter, pi-subagents, gzmo-pi, hsp-pi
        │
        ├─ L1 shell bridge ── scripts/pi-gzmo-memory.sh
        │       stable GZMO_SESSION_ID → Redis scratch survives turns
        │
        ├─ L2 MCP (stdio) ── ~/.pi/agent/mcp.json   ← adapter reads THIS
        │       gzmo-living → CT101 mcp-serve (goal C)
        │       gzmo-memory → product ~/.gzmo (goal A) — separate name
        │       memory      → Neo4j MCP (living graph; password from .env)
        │
        └─ Cognition ── OpenRouter / Prime (Pi models list)
                │
                ▼
         GZMO PlatformMemory (CT101 daemon or local product)
                │
     ┌──────────┼──────────────┬────────────────────┐
     ▼          ▼              ▼                    ▼
 Redis      Distill queue   Vault/honeypot      Neo4j / Qdrant
 scratch    gzmo:distill:   /opt/gzmo           (living only)
 gzmo:scratch: pending
 {main|sub|orch}
     │
     └─ (LOST on living HEAD) Headroom-inspired CCR
        gzmo:ccr:* + gzmo_retrieve_context
        → only on branch feat/context-compress-headroom
```

---

## Redis: what subagents actually used

### GZMO-core (the real Redis cache)

Source: `gzmo-core/src/memory/scratch.rs`, [scratch-redis.md](../ct101-systems/50-memory-data-plane/scratch-redis.md).

| Scope | Redis key shape | Who writes |
|-------|-----------------|------------|
| Main session | `gzmo:scratch:main:{session_id}` | Chat / MCP turn |
| **Subagent** | `gzmo:scratch:sub:{session_id}:{task_id}` | GZMO `SubagentRunner` / `delegate_task` |
| Orchestrator | `gzmo:scratch:orch:{job}:{step}` | Daemon jobs |
| Distill | `gzmo:distill:pending` (+ file fallback) | Session close / Synapse `session_end` |
| Embed cache | `gzmo:embed:*` | Embedding path |

**Important distinction:**

- **GZMO subagents** (`[subagent]` / `delegate_task` in Rust) → Redis `ScratchScope::Sub`.
- **Pi `npm:pi-subagents`** → Pi child agents with local artifacts under `~/.pi-subagents/` — they do **not** speak Redis to GZMO unless you also call GZMO MCP with a stable session id.

For Pi turns to share hot scratch across messages:

```bash
# Always go through the bridge (creates/reuses session id)
./scripts/pi-gzmo-memory.sh prep "your query"
# or MCP: gzmo_memory_turn_start → search → recall_pull
```

Calling bare `gzmo memory …` without a stable `GZMO_SESSION_ID` drops Redis continuity.

### Synapse → distill

Pi/session end can enqueue distill jobs onto Redis so CT101 daemon metabolizes the transcript without Pi waiting on the LLM distill. That is why Redis felt like “Pi’s cache” — the **queue and scratch** live in GZMO; Pi is just the producer.

---

## Headroom (what got lost)

External project: [headroomlabs-ai/headroom](https://github.com/headroomlabs-ai/headroom) — **not cloned** on this host.

What we actually built: an **in-tree Rust port** inspired by Headroom (Apache-2.0 NOTICE), on branch:

```text
origin/feat/context-compress-headroom
  commits: a373d33 → df078f3 → c909277
  module: gzmo-core/src/context_compress/{mod,logs,json,ccr}.rs
```

| Piece | Role |
|-------|------|
| Content router | Detect logs / JSON / plain → compress for context window |
| **CCR** | Cache full text in Redis `gzmo:ccr:{session}:{hash}`; inject `[ccr:<hash> — gzmo_retrieve_context to expand]` |
| MCP | `gzmo_retrieve_context` to expand a hash |
| Bench | `scripts/compression-bench/benchmark_headroom.py` (Python Headroom for comparison only) |

**Not on living HEAD today** — `context_compress/` is absent from `main`. Re-land details: [HEADROOM_CCR.md](../HEADROOM_CCR.md).

**Explicit non-goals (from Phase 3 handoff):** no permanent `headroom proxy` in production; no compressing vault/honeypot writes; no replacing Qdrant/RRF.

---

## Pi packages (live inventory pattern)

Typical `~/.pi/agent/settings.json` packages (workstation, recovered 2026-07-19):

- `npm:pi-mcp-adapter` — loads MCP from **`mcp.json`**, not from `settings.json` alone  
- `npm:pi-subagents` — concurrency 2, roles scout/planner/reviewer/worker  
- `npm:gzmo-pi` and/or `git:github.com/maximilianwruhs-cyber/gzmo-pi` — product UX  
- `npm:hsp-pi` — audio sidecar (unrelated to Redis memory)

Merge snippet for subagents: [pi-settings-subagents.snippet.json](../pi-settings-subagents.snippet.json).

CT101 Pi install lessons (`~/.pi/agent/HANDOFF_CT101_PI.md`):

1. Adapter **requires** `mcp.json`  
2. `--no-extensions` kills MCP  
3. Fake `--extension` paths break cycles  
4. Hardcoded old home paths (`maximilian-wruhs`, `survey_GZMO`) drift on every host move  

---

## Why official Pi updates kept breaking you

Customization lived in **agent home + shell wrappers + discovery flags**, not behind a stable extension API.

| Fragility | Failure mode |
|-----------|--------------|
| Dual config | Packages in `settings.json`, servers in `mcp.json` — update one, forget the other |
| Product vs living collision | Same Pi home; living must be **`gzmo-living`**, product **`gzmo-memory`** — never one name for both |
| CLI flags | Discovery scripts with `--no-extensions` / bad `--extension` |
| Path drift | Scripts baked to old usernames / clone roots |
| Package double-list | Both `npm:gzmo-pi` and git gzmo-pi → version fights |
| Headroom expectations | Prompts/`[ccr:…]` tools after CCR fell off living binary |
| No pin policy | Upstream Pi + adapter bump without a smoke checklist |

**Clean rule:** treat Pi upgrades as a **release**: run [PI_UPGRADE_RUNBOOK.md](PI_UPGRADE_RUNBOOK.md); never hand-edit only one of settings/mcp/scripts.

---

## Correct living attach (today)

```bash
# From GZMO clone — wires CT101 living memory + Neo4j into Pi/Cursor
bash scripts/install-shared-mcp.sh

# Smoke
bash scripts/ct101-living-smoke.sh
# In Pi: gzmo_memory_status  → vault under /opt/gzmo, ~60k facts
```

Product laptop path (separate):

```bash
bash scripts/install-product-mcp.sh   # or install-gzmo.sh
bash scripts/product-readiness-gate.sh
```

Do **not** put living CT101 under the product name `gzmo-memory`. Use **`gzmo-living`** + **`gzmo-memory`** side by side when both attaches are needed.

---

## Recovery map (where knowledge lived)

| Artifact | Location |
|----------|----------|
| Living MCP attach | `docs/PI_GZMO_MEMORY_INTEGRATION.md` |
| Operator demotion of Pi | `docs/OPERATOR_FRONTEND_DECISION.md` |
| Redis scratch scopes | `docs/ct101-systems/50-memory-data-plane/scratch-redis.md` |
| CT101 Pi install scars | `~/.pi/agent/HANDOFF_CT101_PI.md` |
| Headroom CCR implementation | `git show origin/feat/context-compress-headroom:…` |
| Headroom plan (missing on this host) | was `~/.cursor/plans/headroom_ideas_for_gzmo_b349ab4c.plan.md` under old home |

---

## Security note

If Neo4j (or any) passwords were ever pasted into `~/.pi/agent/settings.json` or committed fragments, **rotate them** and keep secrets only in `.env` / `install-shared-mcp.sh` pull-from-CT101. Never commit passwords into `docs/`.
