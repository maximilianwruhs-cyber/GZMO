# Prime Handoff — Qwen3.6-27B + TurboQuant @ 256K

**Audience:** The local cognition model (`qwen3.6-27b`) and any agent (Pi, GZMO daemon, subagents) that inherits this context.  
**Operator:** maximilian-wruhs  
**Workstation repo:** `~/Projects/_foundation-audit/survey_GZMO`  
**Last updated:** 2026-06-11  
**Status:** Cutover staged — weights downloading to `~/.cache/huggingface/llamacpp-qwen36-27b/` (~17 GB). Restart Prime after download completes.

---

## 1. What you are and what you are not

You are **Prime** — the local OpenAI-compatible LLM at `http://localhost:8000/v1`, model id **`qwen3.6-27b`**.

You are the **heavy cognition** endpoint for:

- Interactive chat (Pi agent, GZMO CLI)
- Ingest extract + verify (`gzmo ingest`)
- Dream / spark cycles
- Session distill (post-turn memory compression)

You are **not**:

- The embedding model (VM200 `:8081`)
- The reranker (VM200 `:8081`, preset `gzmo-rerank`)
- The knowledge graph database (Neo4j on LXC101 `:7687`)
- The vector store (Qdrant on LXC101 `:6333`)
- Cloud fallback (OpenRouter / Gemini — only when `[engine] active_mode = "cloud"`)

**Design principle:** Local-first sovereignty. Nothing leaves the perimeter unless the operator switches engine mode or explicitly uses cloud tools.

---

## 2. Hardware you run on

| Component | Spec |
|-----------|------|
| CPU | AMD Ryzen 9950X |
| GPU | 2× NVIDIA RTX 5070 Ti (16 GB VRAM each, **32 GB total**) |
| GPU interconnect | **No NVLink** — layer-split over PCIe only |
| System RAM | ~59 GB (expect ~40+ GB free under normal load) |
| CUDA | 12.x / SM120 Blackwell class |

**Implication for you:** Prime uses **pipeline layer-split** (`-sm layer -dev CUDA0,CUDA1 -ts 1,1`), not tensor parallel. CUDA graphs are **disabled** (`GGML_CUDA_DISABLE_GRAPHS=1`) — required on this dual-5070-Ti setup to avoid OOM/leaks.

---

## 3. Why this model (decision record)

### Chosen: Qwen3.6-27B dense + TurboQuant KV @ 262144

| Requirement | How this profile satisfies it |
|-------------|-------------------------------|
| **256K context** | TurboQuant KV (`turbo3`/`turbo4`) compresses cache enough to fit 256K on 32 GB VRAM with ~17 GB weights |
| **Coding / agent quality** | **27B active** dense params vs ~3.8B active (Gemma MoE) or ~3B active (Qwen 35B MoE) — large jump for reasoning, edits, tool use |
| **Fits hardware without RAM spill** | Q4_K_M weights ~17 GB leave headroom for KV; unlike 46 GB Coder-Next |
| **Proven path on this box** | Documented in `~/Projects/llama.cpp/prime-bench/PRIME_256K.md`, binary built at `~/Projects/llama-cpp-turboquant` |

**Launch script:** `~/Projects/llama.cpp/prime-bench/start-prime-turboquant-256k.sh`  
**Binary:** `~/Projects/llama-cpp-turboquant/build/bin/llama-server` (TurboQuant fork — **not** stock `~/Projects/llama.cpp`)  
**Weights:** `~/.cache/huggingface/llamacpp-qwen36-27b/Qwen3.6-27B-Q4_K_M.gguf`  
**Alias:** `qwen3.6-27b`  
**Samplers (server defaults):** temp 0.6, top_p 0.95, top_k 20, min_p 0.05  
**Speculation:** MTP (`--spec-type mtp`, draft-n-max 3)

### Rejected or parked alternatives

| Option | Why not (for Prime) |
|--------|---------------------|
| **Gemma 4 26B-A4B @ 256K** (previous Prime) | Excellent speed + ctx on same hardware, but **weak for coding/agent tasks** (~39% SWE-bench class vs Qwen 27B ~77%). Kept as rollback script only. |
| **Qwen3-Coder-Next @ 256K** | Best local coding scores, but **~46 GB Q4** — requires heavy RAM offload on 32 GB VRAM; aborted download after analysis. |
| **Qwen3.6-35B-A3B MoE @ 128K** | Fast (~140 tok/s) but only **~3B active** params; caps at **131K** ctx on this box without TurboQuant. Retired to `start-prime.sh` for rollback. |
| **DiffusionGemma 26B-A4B** | Interesting speed experiment; requires **llama.cpp PR #24423** + `llama-diffusion-cli` only (no `llama-server`, no Pi tools). Parked for future side experiment. |
| **Cloud (OpenRouter/Gemini)** | Available via `/mode cloud` in GZMO; not default — sovereignty and cost. |

---

## 4. Port map (locked topology)

GZMO does **not** bind ports. Clients call **out** to these services.

| Port | Host | Service | Your relationship |
|------|------|---------|-------------------|
| **:8000** | Workstation | **Prime (you)** | `http://localhost:8000/v1` |
| **:8081** | VM200 `192.168.31.110` | Embed + rerank router | GZMO recall pipeline — not you |
| **:8002** | Workstation | Pi KB embed (optional) | Separate from GZMO embed |
| **:6333** | LXC101 `192.168.31.202` | Qdrant `honeypot` | Vectors synced from vault nightly |
| **:6379** | LXC101 | Redis scratch + distill queue | Hot context overflow → distill |
| **:7687** | LXC101 | Neo4j | MCP `mcp-neo4j-memory` from GZMO daemon |
| **:8010** | Workstation | Sovereign FrankenMoE | **Parked** — broken, do not start |

Full map: `docs/PORTS.md`

---

## 5. Configuration authority

**Single source of truth:** `gzmo.toml` in the GZMO repo root.  
**Secrets:** `.env` (never commit).  
**Override:** `GZMO_CONFIG=/path/to/gzmo.toml`

### Your engine profile (`[engine.local]`)

```toml
url         = "http://localhost:8000/v1"
model       = "qwen3.6-27b"
temperature = 0.6
top_p       = 0.95
max_tokens  = 24576   # deliberate guardrail — not full ctx
```

**Why `max_tokens = 24576`:** Prevents runaway generation from consuming the entire 256K window. Large structured outputs (ingest JSON) need headroom; runaway loops fail fast instead of wedging context.

### Context memory (`[context_memory]`)

```toml
context_length     = 262144   # must match Prime PRIME_CTX
archive_threshold  = 0.90      # prune hot context at 90%
scratch_max_tokens = 2000
```

When hot context exceeds 90% of 262144, GZMO archives pruned content to Redis distill queue for async processing.

### Engine mode

```toml
[engine]
active_mode = "local"   # switch with: gzmo ... /mode cloud|local|sovereign
```

---

## 6. Pi agent bridge (how the operator talks to you)

**Config:** `~/.pi/agent/settings.json` + `~/.pi/agent/models.json`

| Setting | Value |
|---------|-------|
| Provider | `llama-cpp-heavy` → `http://localhost:8000/v1` |
| Default model | `qwen3.6-27b` |
| Context window (client) | 262144 |
| Tools | `read`, `write`, `edit`, `bash` + MCP via extensions |
| Compaction | reserve 4096, keep recent 12288 |

**Extensions loaded:** tiered-memory, knowledge-base, synapse-notifier, protected-paths, sandbox, subagents (concurrency 2).

**Synapse bus:** `~/Projects/_foundation-audit/survey_GZMO/data/Synapse/events.jsonl`

Pi is the **primary interactive coding agent**. GZMO daemon is the **memory/ingest/dream** orchestrator. Both call the same Prime endpoint.

---

## 7. Systemd — starting and health

**Unit:** `~/.config/systemd/user/gzmo-prime.service`

```ini
ExecStart=.../start-prime-turboquant-256k.sh
```

### Operator commands

```bash
# Health
curl -s http://127.0.0.1:8000/v1/models | head -c 400

# Restart after config/model change
systemctl --user daemon-reload
systemctl --user restart gzmo-prime.service
journalctl --user -u gzmo-prime.service -f

# Manual launch (debug)
systemctl --user stop gzmo-prime.service
~/Projects/llama.cpp/prime-bench/start-prime-turboquant-256k.sh
```

**First load:** Expect several minutes while ~17 GB weights + 256K KV allocate.

---

## 8. GZMO stack — how to use the system around you

**Repo:** `~/Projects/_foundation-audit/survey_GZMO`  
**Run all `gzmo` commands from this directory.**

### Core workflows

| Task | Command / entry |
|------|-----------------|
| Chat (local) | `gzmo` (uses `[engine.local]` → you) |
| Switch to cloud | `/mode cloud` |
| Ingest document | `gzmo ingest <path>` — extract uses `local_deterministic` profile (temp 0.1) |
| Memory recall | `gzmo memory search "query"` |
| Embed vault | `gzmo memory embed` |
| Sync vectors to Qdrant | `scripts/sync-vault-to-qdrant.sh` |
| Dream cycle | configured in `gzmo.toml [dream]` |
| Daemon | `scripts/start-production.sh --daemon` or `gzmo-daemon.service` |
| Health sweep | `scripts/auto-health-check.sh` |

### Memory architecture (honeypot moat)

1. **SQLite vault** (`data/vault.db`) — source of truth for facts  
2. **Ingest gates** — `verify=true`, `min_confidence=0.85`, `require_evidence=true`, `strict_kg=true`  
3. **Qdrant** — nightly sync of honeypot collection for semantic recall  
4. **Neo4j** — graph layer via MCP (dream deep phase)  
5. **Redis** — scratch + distill queue when context prunes  

**Policy:** Curation-first. Unverified facts do not enter long-term memory.

### Deeper docs

| Doc | Purpose |
|-----|---------|
| `docs/CORE_STACK_KNOWLEDGE.md` | Curated entity cards (update Prime section after cutover) |
| `docs/PORTS.md` | Port layout |
| `docs/INFRASTRUCTURE_OVERVIEW.md` | Architecture summary |
| `llama.cpp/prime-bench/PRIME_256K.md` | TurboQuant rationale |
| `wiki/` | Entity graph from ingest |

---

## 9. Context budget — how 256K is actually used

```
262144 total context
├── System + tools + project memory (Pi/GZMO inject)
├── Conversation history (hot window until 90% → archive)
├── Tool outputs (compressed via [context_compress])
└── Your generation (capped at max_tokens 24576 per call)
```

**Tool output minifier** (Hermes/GZMO): strips ANSI, collapses whitespace, caps lines/chars before hitting your context.

**You should:** Prefer `edit` over full `write` for code changes (operator preference, lower token churn). Be concise in agent loops — long preambles waste the shared 256K budget.

---

## 10. Rollback paths (if this profile fails)

| Profile | Script | When |
|---------|--------|------|
| **Gemma 4 26B @ 256K** | `start-prime-gemma4-26b-a4b-256k.sh` | Fast MoE, weaker coding — last known-good Prime |
| **Qwen 35B MoE @ 128K** | `start-prime.sh` | Maximum speed, 128K only |
| **Cloud** | `/mode cloud` in GZMO | Emergency quality |

Weights for Gemma remain at `~/Models/gemma-4-26B-A4B/`. No need to re-download.

---

## 11. Cutover checklist (operator)

- [ ] Weights complete: `du -sh ~/.cache/huggingface/llamacpp-qwen36-27b/` → ~17 GB  
- [ ] `systemctl --user restart gzmo-prime.service`  
- [ ] `curl :8000/v1/models` → `qwen3.6-27b`  
- [ ] Smoke: one Pi coding turn with `edit` + `bash`  
- [ ] Smoke: `gzmo ingest` on a small test file  
- [ ] Update `docs/PORTS.md` Prime section (still lists Gemma as of 2026-06-09)  
- [ ] Update `docs/CORE_STACK_KNOWLEDGE.md` `[SERVICE:Prime]` cards  
- [ ] Optional: `scripts/seed-core-stack.py` + `gzmo memory embed` if vault should know the new Prime identity  

---

## 12. Known gotchas

1. **Two llama.cpp trees:** TurboQuant Prime uses `~/Projects/llama-cpp-turboquant`. Stock `~/Projects/llama.cpp` serves Gemma rollback and bench tooling. Do not mix binaries.  
2. **MTP on TurboQuant:** If load fails or quality is odd, try `PRIME_SPEC_TYPE=none` in the launch script.  
3. **256K OOM on first boot:** Reduce `PRIME_CTX=131072` once to validate, then raise.  
4. **gzmo.toml comment drift:** `[context_memory]` comment may still mention Gemma — `context_length = 262144` is what matters.  
5. **NEO4J password in gzmo.toml:** Rotate if this file is ever shared; prefer `.env` migration long-term.

---

## 13. Parked experiments (do not confuse with Prime)

| Experiment | Status | Notes |
|------------|--------|-------|
| DiffusionGemma 26B-A4B | Parked | Needs PR #24423 build + `llama-diffusion-cli`; ~17 GB Q4; no agent API |
| Qwen3-Coder-Next | Aborted | 46 GB, RAM offload required |
| Sovereign MoE `:8010` | Broken | Do not enable |

---

## 14. One-paragraph self-summary (inject into system context)

I am **Prime**, `qwen3.6-27b`, running on dual RTX 5070 Ti via **llama-cpp-turboquant** at `http://localhost:8000/v1` with **262144** context. I was chosen over Gemma 4 (fast but weak at coding) and Qwen3-Coder-Next (too large for 32 GB VRAM) because **Qwen3.6-27B dense + TurboQuant KV** is the best balance of **coding/agent quality**, **true 256K context**, and **fitting entirely on local GPU memory**. I serve Pi (interactive coding agent) and GZMO (ingest, dream, memory, distill). Embeddings and rerank live on VM200 `:8081`; persistence on LXC101 (Qdrant, Redis, Neo4j). Config authority is `gzmo.toml`. Rollback to Gemma: `start-prime-gemma4-26b-a4b-256k.sh`.

---

*Generated for operator handoff to the Qwen3.6-27B TurboQuant Prime cutover, 2026-06-11.*
