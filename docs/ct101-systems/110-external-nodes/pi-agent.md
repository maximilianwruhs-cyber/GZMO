# Pi Agent — Optional Frontend Node

**Source:** `~/.pi/agent/`, [PI_FRONTEND_SPLIT.md](../../PI_FRONTEND_SPLIT.md), [OPERATOR_FRONTEND_DECISION.md](../../OPERATOR_FRONTEND_DECISION.md)  
**Parent:** [110-external-nodes/SYSTEM.md](./SYSTEM.md)

---

## Capability

**Pi** (`pi` CLI, config under `~/.pi/agent/`) is an optional interactive frontend on the **workstation** — not the canonical operator UI (that is `gzmo chat`). Its primary CT101 integration is **Synapse telemetry**: `session_end` events appended to CT101's `events.jsonl`, consumed by `[synapse_pull]` for session distill.

Discovery automation also drives Pi heavily via `pi-mentor-discovery-cycle.sh` on CT101 — separate from the desktop Pi REPL.

---

## How it works

### Topology

| Component | Host | Role |
|-----------|------|------|
| Pi agent REPL | Workstation `~/.pi/agent/` | Optional human chat |
| Pi discovery subprocess | CT101 `gzmo_skills` | Infrastructure mentor dialogue (OpenRouter) |
| Synapse bus | CT101 `/opt/gzmo/data/Synapse/events.jsonl` | Append-only event log |
| Synapse pull | CT101 daemon `synapse_reader.rs` | Polls `session_end` → distill queue |

From [PI_FRONTEND_SPLIT.md](../../PI_FRONTEND_SPLIT.md):

> Session distillation: CT101 `[synapse_pull]` polls `data/Synapse/events.jsonl` and runs `gzmo distill` on Pi `session_end` — Pi events must reach the Synapse bus (workstation → CT101 path).

### Operator frontend decision

[OPERATOR_FRONTEND_DECISION.md](../../OPERATOR_FRONTEND_DECISION.md) demotes Pi to **optional auxiliary**:

1. Does not own session distill authority (daemon + `session_end` → queue wins)
2. Synapse `session_end` events are optional telemetry, not primary lifecycle
3. Canonical UI: `gzmo chat` on workstation

### Discovery vs REPL

| Path | Trigger | Pi role |
|------|---------|---------|
| **Discovery cycle** | auto-socratic / timer | Headless `pi -p` with infra-discovery prompts on CT101 |
| **Operator REPL** | Human `pi` on workstation | Interactive; may emit `session_end` to Synapse |

### Config artifacts

| Path | Purpose |
|------|---------|
| `~/.pi/agent/settings.json` | Pi agent settings |
| `~/.pi/agent/mcp.json` | MCP server paths (workstation) |
| `~/.pi/agent/MEMORY_CONTEXT.md` | Pi memory context (see PI_GZMO_MEMORY_INTEGRATION.md) |
| `~/.pi/agent/extensions/*.ts` | Optional Pi extensions |

---

## Interfaces

| Interface | Value |
|-----------|-------|
| Synapse file | `/opt/gzmo/data/Synapse/events.jsonl` (CT101) |
| Event type | `session_end` (and discovery remediation via `emit_synapse_event`) |
| Pull config | `[synapse_pull]` in `gzmo.toml` on CT101 |
| Discovery cwd | `PI_DISCOVERY_CWD` → `gzmo_skills` root on CT101 |
| MCP bridge | Shared MCP packages; paths corrected per host |

---

## THINKING nodes

> **THINKING — pi-agent:split authority**
> - *Reviewed:* Pi REPL optional; daemon distill queue is authoritative.
> - *Insight:* Prevents split-brain memory when Pi and gzmo chat both active.
> - *Risk / limitation:* Operators may assume Pi sessions auto-distill immediately.
> - *Enhancement:* Banner in Pi when Synapse path not wired to CT101. [CT101-safe]

> **THINKING — pi-agent:session_end path**
> - *Reviewed:* Events must physically reach CT101 JSONL — workstation → sync → CT101.
> - *Insight:* Discovery scripts use `emit_synapse_event` with explicit `GZMO_ROOT/data` path.
> - *Risk / limitation:* Wrong `GZMO_ROOT` writes events to wrong host's file (Jul 10 class bug).
> - *Enhancement:* Synapse writer health check in discovery post-distill hooks. [CT101-safe]

> **THINKING — pi-agent:discovery headless**
> - *Reviewed:* CT101 runs `pi` subprocess with timeouts and `pkill` cleanup.
> - *Insight:* Pi is infrastructure probe tool, not just human UI.
> - *Risk / limitation:* Residual Pi processes after watchdog failure (cleanup uses `pkill -9`).
> - *Enhancement:* cgroup-scoped Pi runner per `session_id`. [CT101-safe]

---

## Advancement

| CT101 | GZMO-next |
|-------|-----------|
| Pi discovery on CT101 | Lab mentor piece with beat-gate report schema |
| Pi REPL optional | `gzmo chat` only operator surface |
| File-based Synapse | Redis stream bus shared across nodes |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Verify Synapse path in discovery post-distill hook | [CT101-safe] |
| 2 | Document Pi vs gzmo chat authority in operator onboarding | [CT101-safe] |
| 3 | cgroup-isolated Pi discovery subprocess | [CT101-safe] |
| 4 | Retire Pi REPL from hot path when gzmo chat parity complete | [GZMO-next] |
| 5 | Unified session lifecycle events schema | [GZMO-next] |
