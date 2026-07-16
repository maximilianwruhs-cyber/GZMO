# GZMO Platform architecture

**Status:** Accepted (2026-06-04); operator frontend updated **2026-07-10** — see [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md)  
**Supersedes for operator UX:** [INFRASTRUCTURE_OVERVIEW.md](./INFRASTRUCTURE_OVERVIEW.md) §10.1 multi-UI “Hindsight target” — one daily frontend: **`gzmo_cli`**.

---

## 1. One sentence

**GZMO Platform** = living `config/gzmo.toml` spine + hot/cold memory + overnight metabolism. **Three surfaces:** `gzmo chat`, `gzmo serve`, `gzmo memory mcp`. Pi/Cursor attach via MCP only; see [ADR-0003-one-instance-metabolism.md](./ADR-0003-one-instance-metabolism.md) and [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md).

---

## 2. Layers

```mermaid
flowchart TB
  subgraph frontend [Frontend optional instance pi-rust]
    Client[Operator client]
  end
  subgraph platform [GZMO Platform]
    TomlSpine[Config spine gzmo.toml]
    Hot[Hot runtime scratch archive subs]
    Cold[Cold stores vault honeypot]
    Fabrik[Daemon ingest dream spark]
  end
  subgraph infra [Infrastructure]
    Prime[Prime :8000]
    VM200[Embed rerank VM200]
    Redis[Redis LXC101]
  end
  Client -->|memory tools future| Hot
  Client -->|completions| Prime
  Hot --> Cold
  TomlSpine --> Hot
  TomlSpine --> Cold
  TomlSpine --> Fabrik
  Fabrik --> Cold
```

---

## 3. Config spine (`gzmo.toml`)

Single authority for all clients and the daemon.

| Section | Role |
|---------|------|
| `[engine]` / `[engine.local]` / `[engine.cloud]` | Cognition routing — Prime URL, model, mode |
| `[routing]` | `TaskKind` → engine profile (dream, distill, chat, …) |
| `[memory]` | Vault paths, episodic directory |
| `[embeddings]` / `[rerank]` | Semantic recall |
| `[redis]` | Scratch + distill queue |
| `[context_memory]` | Archive @ 90%, hot budget |
| `[subagent]` | `delegate_task` governor (max 2, summary cap) |
| `[agent]` | Tool iteration limits |

Frontends **read** this file; they do not fork parallel engine/memory config.

---

## 4. Hot vs cold

| Mode | Mechanism | Lifetime |
|------|-----------|----------|
| **Hot** | `ScratchService`, `prune_with_archive`, `[RECALL]`, distill enqueue | Per turn / session |
| **Cold** | Vault → honeypot → Qdrant; daemon promote/dream/spark | Days+ |

Implementation: [`agent_session.rs`](../gzmo-core/src/agent_session.rs), [`agent_loop.rs`](../gzmo-core/src/agent_loop.rs), [`scratch.rs`](../gzmo-core/src/memory/scratch.rs).

---

## 5. `AgentSession` (platform API)

Any frontend binds hot memory through:

| Method | When |
|--------|------|
| `AgentSession::new_main` | Session start |
| `turn_start()` | New user message — clear scratch, cancel subs |
| `loop_config(...)` | Before `run_agent_loop` |
| `set_session_id` | `/new`, `/resume`, `/load` |

Legacy harness: [`chat.rs`](../gzmo-cli/src/chat.rs) calls `AgentSession`; chaos/skills/slash stay in the harness only.

**P1 (2026-06-04):** CLI + tool names for frontends:

```bash
gzmo memory turn-start              # new operator turn — clear scratch
gzmo memory search "<query>"        # gzmo_memory_search
gzmo memory recall                  # gzmo_memory_recall_pull → [RECALL] block
gzmo memory status [--json]
export GZMO_SESSION_ID=<id>         # stable session across commands
```

Rust: [`platform_memory.rs`](../gzmo-core/src/platform_memory.rs) — `GzmoMemorySearchTool`, `GzmoMemoryStatusTool`, `GzmoMemoryRecallPullTool` for in-process agents.

---

## 6. Operator clients

| Binary | Role today |
|--------|------------|
| **`gzmo` / `gzmo chat`** | **Primary operator REPL** — `AgentSession`, tools, chaos skills |
| **`gzmo assemble`** | Little Tools Lab recipe runner |
| **`gzmo memory *`** | Platform memory API for scripts and integrations |
| **`gzmo tui` / `--repl`** | Legacy TUI; `memory: None` — deprecate or wire in P4 |
| **Pi** (`~/.pi/agent/`) | Optional; not product UI — [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md) |

---

## 7. Concrete frontend instance

**Current operator client:** **`gzmo_cli`** on the workstation (default `gzmo` → chat REPL, Prime `:8000`, shared vault/honeypot, `gzmo assemble` for lab recipes).

**Canonical:** [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md)

**Pi (auxiliary only):** [PI_OPERATOR_GUIDE.md](./PI_OPERATOR_GUIDE.md) — historical; use only via `gzmo memory` bridge, not as assembly authority.

---

## 8b. Little Tools Lab (GZMO-next)

**CT101** runs **standalone legacy** GZMO — no per-loop lab graft. See [CT101_BOUNDARY.md](./CT101_BOUNDARY.md).

**Little Tools Lab** hosts 46 puzzle pieces and builds **GZMO-next** as a full assembly on the workstation. `beat-gate.sh` uses CT101 behavior as a **reference baseline**, not a promotion trigger.

**Canonical:** [LAB_TREATMENT.md](../../little-tools-lab/docs/LAB_TREATMENT.md)

---

## 8. Roadmap

| Phase | Deliverable |
|-------|-------------|
| **P0** ✅ | `AgentSession` + arch doc (this file) |
| **P1** ✅ | `gzmo memory *` CLI + `gzmo_memory_*` tool types |
| **P2** ✅ | Orchestrator hot scope `orch:{job}:{step}` + `from_memory_config` |
| **P3** ✅ | `scripts/pi-gzmo-memory.sh` + [PI_GZMO_MEMORY_INTEGRATION.md](./PI_GZMO_MEMORY_INTEGRATION.md) |
| **P4** ✅ | Operator frontend = `gzmo_cli` — [OPERATOR_FRONTEND_DECISION.md](./OPERATOR_FRONTEND_DECISION.md) |
| **P5** ✅ | Chat session_end → distill enqueue; slash → `gzmo assemble` |

---

## 9. References

| Doc | Role |
|-----|------|
| [MEMORY_ARCHITECTURE_SPEC.md](./MEMORY_ARCHITECTURE_SPEC.md) | Cold layers, tool names |
| [SCRATCH_MEMORY_VERIFY.md](./SCRATCH_MEMORY_VERIFY.md) | Static verify (Handover I) |
| [SCRATCH_TUI_GAP.md](./SCRATCH_TUI_GAP.md) | Why TUI is not wired |
| [INFRASTRUCTURE_OVERVIEW.md](./INFRASTRUCTURE_OVERVIEW.md) | Ops canonical |
| [PLATFORM_BASELINE_STATUS.md](./PLATFORM_BASELINE_STATUS.md) | Green gate + label |
| [LAB_TREATMENT.md](../../little-tools-lab/docs/LAB_TREATMENT.md) | Lab incubator model, promotion ladder |
| [CT101_BOUNDARY.md](./CT101_BOUNDARY.md) | CT101 standalone; no loop swap |

---

*End.*
