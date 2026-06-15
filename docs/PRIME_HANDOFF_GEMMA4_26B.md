# Prime Handoff — Gemma 4 26B-A4B MoE QAT @ 256K

**Audience:** The local cognition model (`gemma-4-26b-a4b-it`) and any agent (Pi, GZMO daemon, subagents) that inherits this context.  
**Operator:** maximilian-wruhs  
**Workstation repo:** `~/Projects/_foundation-audit/survey_GZMO`  
**Last updated:** 2026-06-11  
**Status:** **Production** — live on `:8000` via `gzmo-prime.service` → `start-prime-gemma4-26b-a4b-256k.sh`.

---

## 1. What you are and what you are not

You are **Prime** — the local OpenAI-compatible LLM at `http://localhost:8000/v1`, model id **`gemma-4-26b-a4b-it`**.

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

**Architecture:** Gemma 4 **26B-A4B MoE** with **~3.8B active parameters** per token (4-of-N experts). Total weights ~25B params; you are fast because most layers are sparse.

---

## 2. Hardware you run on

| Component | Spec |
|-----------|------|
| CPU | AMD Ryzen 9950X |
| GPU | 2× NVIDIA RTX 5070 Ti (16 GB VRAM each, **32 GB total**) |
| GPU interconnect | **No NVLink** — layer-split over PCIe only |
| System RAM | ~59 GB |
| CUDA | 12.x / SM120 Blackwell class |

**Implication for you:** Prime uses **pipeline layer-split** (`-sm layer -dev CUDA0,CUDA1 -ts 1,1`), not tensor parallel. CUDA graphs are **disabled** (`GGML_CUDA_DISABLE_GRAPHS=1`) — required on this dual-5070-Ti setup; sweep showed no benefit and community reports graph corruption risk on this rig.

**VRAM at 256K (validated):** ~11.5 GB + ~12.4 GB per GPU after load (~8 s cold start).

---

## 3. Why this model (decision record)

### Production choice: Gemma 4 26B-A4B QAT @ 262144

| Requirement | How this profile satisfies it |
|-------------|-------------------------------|
| **True 256K context on 32 GB VRAM** | ~14 GB QAT weights + f16 KV fits 262144 without RAM spill or TurboQuant fork |
| **Proven daily driver** | Benchmarked champion profile; loads reliably on stock `llama.cpp` with Gemma4 MTP (PR #23398+) |
| **Speed** | ~185–212 tok/s (mtp-bench / llama-bench TG128) — best throughput of any Prime profile tested here |
| **Stability** | No hybrid-arch load failures (unlike Qwen3.6-27B on turboquant fork, 2026-06-11) |
| **Long-context workloads** | Ingest chunks, dream REM, archive-aware chat benefit from 256K vs legacy 128K Qwen MoE Prime |

**Launch script:** `~/Projects/llama.cpp/prime-bench/start-prime-gemma4-26b-a4b-256k.sh`  
**Binary:** `~/Projects/llama.cpp/build/bin/llama-server` (**stock** llama.cpp — **not** `llama-cpp-turboquant`)  
**Main weights:** `~/Models/gemma-4-26B-A4B/gemma-4-26B-A4B-it-qat-UD-Q4_K_XL.gguf` (~14 GB)  
**MTP assistant:** `~/Models/gemma-4-26B-A4B/gemma-4-26B-A4B-it-assistant-Q2_K.gguf` (~278 MB)  
**Chat template:** `~/Projects/llama.cpp/models/templates/google-gemma-4-31B-it-interleaved.jinja`  
**Alias:** `gemma-4-26b-a4b-it`  
**Context:** `262144` (256K)  
**KV cache:** `f16` / `f16` (not TurboQuant — see §13)  
**Samplers (server defaults):** temp 0.55, top_p 0.9, top_k 64, min_p 0.01  
**Speculation (champion):** `draft-mtp,ngram-mod` stacked, draft-n-max 3, ngram-mod 24/48/64  
**Flags:** `--jinja`, `--chat-template-file`, `--reasoning off`, `--flash-attn on`

### Sweep champion (2026-06-09, dual RTX 5070 Ti)

| Profile | llama-bench TG128 | mtp-bench mean | Verdict |
|---------|-------------------|----------------|---------|
| **MTP + ngram-mod stacked** | **212.2** | **185.7 tok/s** | **Production winner** |
| baseline nospec | 194.4 | 182.9 | Fallback (`PRIME_SPEC_TYPE=none`) |
| ngram-mod only | 193.8 | 182.3 | No gain alone — reject |
| MTP n=4 alone | 211.4 | 158.0 | Worse real throughput |
| asymmetric q8/q4 KV | 125.5 | — | Reject |
| single GPU | 192.4 | — | OOM at 64K on one card |

Full bench notes: `~/Projects/llama.cpp/prime-bench/GEMMA4_26B_PRIME.md`

### Trade-off you accept

| Strength | Weakness |
|----------|----------|
| Fastest local Prime profile | **Weaker coding / agentic reasoning** than dense 27B+ (~39% SWE-bench class vs ~77% for Qwen3.6-27B) |
| 256K without exotic forks | Only **~3.8B active** params per token — caps reasoning depth vs dense models |
| Reliable load + speculation stack | Clients may send wrong `model` id if configs not synced (see §5) |

### Evaluated but not production (and why)

| Option | Why not (for Prime today) |
|--------|---------------------------|
| **Qwen3.6-27B + TurboQuant @ 256K** | **Target upgrade** — weights downloaded (17 GB) but **load failed** (`missing tensor blk.64.ssm_conv1d.weight`); hybrid SSM arch vs turboquant fork mismatch. Parked until compatible GGUF or fork update. |
| **Qwen3-Coder-Next @ 256K** | Best local coding, but **~46 GB Q4** → RAM offload on 32 GB VRAM; download aborted. |
| **Qwen3.6-35B-A3B MoE @ 128K** | Fast (~140 tok/s) but **~3B active** + **131K ctx cap** on this box; superseded by Gemma 256K. Rollback: `start-prime.sh`. |
| **Gemma 4 31B dense @ 256K** | Needs TurboQuant KV fork; heavier than 26B MoE. |
| **DiffusionGemma 26B-A4B** | Requires llama.cpp PR #24423 + `llama-diffusion-cli` only — no `llama-server`, no Pi. Parked experiment. |
| **Cloud (OpenRouter/Gemini)** | `/mode cloud` available; not default. |

### Why Gemma won over legacy Qwen 35B MoE Prime

Same hardware, **double the context window** (256K vs 128K) with comparable MoE active-param class, plus QAT quality at Q4_K_XL size and a validated speculative stack. Documented in `docs/CORE_STACK_KNOWLEDGE.md` `[SERVICE:Prime]`.

---

## 4. Port map (locked topology)

GZMO does **not** bind ports. Clients call **out** to these services.

| Port | Host | Service | Your relationship |
|------|------|---------|-------------------|
| **:8000** | Workstation | **Prime (you)** | `http://localhost:8000/v1` |
| **:8081** | VM200 `192.168.31.110` | Embed + rerank router | GZMO recall — not you |
| **:8002** | Workstation | Pi KB embed (optional) | Separate from GZMO embed |
| **:6333** | LXC101 `192.168.31.202` | Qdrant `honeypot` | Vectors synced from vault |
| **:6379** | LXC101 | Redis scratch + distill queue | Hot context overflow → distill |
| **:7687** | LXC101 | Neo4j | MCP `mcp-neo4j-memory` from GZMO daemon |
| **:8010** | Workstation | Sovereign FrankenMoE | **Parked** — broken |

Full map: `docs/PORTS.md` (header comments may lag — **you** are the steady-state Prime on `:8000`).

---

## 5. Configuration authority

**Single source of truth:** `gzmo.toml` in the GZMO repo root.  
**Secrets:** `.env` (never commit).  
**Override:** `GZMO_CONFIG=/path/to/gzmo.toml`

### ⚠️ Config drift (fix when convenient)

These files still reference **`qwen3.6-27b`** from the aborted TurboQuant cutover. Prime **actually serves** `gemma-4-26b-a4b-it`. OpenAI-compatible servers often accept any model string, but **align for clarity and logging**:

| File | Field | Should be |
|------|-------|-----------|
| `gzmo.toml` `[engine.local]` | `model` | `gemma-4-26b-a4b-it` |
| `gzmo.toml` `[routing.profiles.local_deterministic]` | `model` | `gemma-4-26b-a4b-it` |
| `gzmo.toml` header comment | `:8000` line | Gemma 4 26B-A4B @ 256K |
| `~/.pi/agent/settings.json` | `defaultModel` | `gemma-4-26b-a4b-it` |
| `~/.pi/agent/models.json` | provider model `id` | `gemma-4-26b-a4b-it` |

### Your engine profile (`[engine.local]` — intended steady state)

```toml
url         = "http://localhost:8000/v1"
model       = "gemma-4-26b-a4b-it"
temperature = 0.6    # server default 0.55; clients may override
top_p       = 0.95
max_tokens  = 24576
```

**Why `max_tokens = 24576`:** Guardrail against runaway generation eating the full 256K window. Ingest JSON and dream extraction need headroom; loops fail fast.

### Context memory (`[context_memory]`)

```toml
context_length     = 262144   # must match PRIME_CTX in launch script
archive_threshold  = 0.90
scratch_max_tokens = 2000
```

### Engine mode

```toml
[engine]
active_mode = "local"   # /mode cloud|local|sovereign in GZMO CLI
```

All background cognition routes to Prime (`[routing]` mappings = `local`).

---

## 6. Pi agent bridge (how the operator talks to you)

**Config:** `~/.pi/agent/settings.json` + `~/.pi/agent/models.json`

| Setting | Value (production) |
|---------|-------------------|
| Provider | `llama-cpp-heavy` → `http://localhost:8000/v1` |
| Default model | `gemma-4-26b-a4b-it` |
| Context window | 262144 |
| Tools | `read`, `write`, `edit`, `bash` + MCP via extensions |
| Compaction | reserve 4096, keep recent 12288 |

**Extensions:** tiered-memory, knowledge-base, synapse-notifier, protected-paths, sandbox, subagents (concurrency 2).

**System prompt append:** `~/.pi/agent/MEMORY_CORE.md`, `MEMORY_ACTIVE.md`

**Synapse bus:** `~/Projects/_foundation-audit/survey_GZMO/data/Synapse/events.jsonl`

Pi is the **primary interactive coding agent**. GZMO daemon is the **memory/ingest/dream** orchestrator. Both call the same Prime endpoint.

**Operator preference:** Use `edit` over full `write` for code changes — lower token churn on the shared 256K budget.

---

## 7. Systemd — starting and health

**Unit:** `~/.config/systemd/user/gzmo-prime.service`

```ini
ExecStart=.../start-prime-gemma4-26b-a4b-256k.sh
Environment=GGML_CUDA_DISABLE_GRAPHS=1
```

(Description line in the unit file may still say Qwen — cosmetic only.)

### Operator commands

```bash
# Health — expect gemma-4-26b-a4b-it, n_ctx 262144
curl -s http://127.0.0.1:8000/v1/models | python3 -m json.tool

# Restart
systemctl --user daemon-reload
systemctl --user restart gzmo-prime.service
journalctl --user -u gzmo-prime.service -f

# Manual launch (debug — stop systemd first)
systemctl --user stop gzmo-prime.service
~/Projects/llama.cpp/prime-bench/start-prime-gemma4-26b-a4b-256k.sh

# Baseline fallback (no speculation)
PRIME_SPEC_TYPE=none PRIME_KV_K=q8_0 PRIME_KV_V=q8_0 \
  ~/Projects/llama.cpp/prime-bench/start-prime-gemma4-26b-a4b-256k.sh
```

**Cold start:** ~8 s load; first 256K prefill is slower than steady decode.

### Download weights (fresh machine)

```bash
cd ~/Projects/llama.cpp/prime-bench
python3 download_model.py gemma4_26b_a4b
python3 download_model.py gemma4_26b_assistant_q2k
```

---

## 8. GZMO stack — how to use the system around you

**Repo:** `~/Projects/_foundation-audit/survey_GZMO`  
**Run all `gzmo` commands from this directory.**

### Core workflows

| Task | Command / entry |
|------|-----------------|
| Chat (local) | `gzmo` → `[engine.local]` → you |
| Switch to cloud | `/mode cloud` |
| Ingest document | `gzmo ingest <path>` — `local_deterministic` (temp 0.1) |
| Memory recall | `gzmo memory search "query"` |
| Embed vault | `gzmo memory embed` |
| Sync vectors | `scripts/sync-vault-to-qdrant.sh` |
| Dream cycle | `gzmo.toml [dream]` |
| Daemon | `scripts/start-production.sh --daemon` |
| Health sweep | `scripts/auto-health-check.sh` |

### Memory architecture (honeypot moat)

1. **SQLite vault** (`data/vault.db`) — source of truth  
2. **Ingest gates** — verify, min_confidence 0.85, evidence required, strict_kg  
3. **Qdrant** — nightly honeypot sync for semantic recall  
4. **Neo4j** — graph via MCP (dream deep phase)  
5. **Redis** — scratch + `gzmo:distill:pending` when context prunes  

**Policy:** Curation-first. Unverified facts do not enter long-term memory.

### Deeper docs

| Doc | Purpose |
|-----|---------|
| `docs/CORE_STACK_KNOWLEDGE.md` | Curated entity cards |
| `docs/PORTS.md` | Port layout |
| `llama.cpp/prime-bench/GEMMA4_26B_PRIME.md` | Bench + champion profile |
| `llama.cpp/prime-bench/PRIME_256K.md` | TurboQuant path (future upgrade) |
| `docs/PRIME_HANDOFF_QWEN36_27B.md` | Parked upgrade handoff |
| `wiki/` | Entity graph from ingest |

---

## 9. Context budget — how 256K is actually used

```
262144 total context
├── System + tools + project memory (Pi/GZMO inject)
├── Conversation history (hot until 90% → Redis distill queue)
├── Tool outputs ([context_compress] minifier)
└── Your generation (max_tokens 24576 per call)
```

**Tool output minifier:** strips ANSI, collapses whitespace, caps lines/chars before hitting context.

**You should:** Be concise in agent loops. Prefer targeted `edit` over rewriting whole files. Long preambles waste shared context.

---

## 10. Upgrade and rollback paths

| Direction | Script / action | When |
|-----------|-----------------|------|
| **Upgrade → Qwen3.6-27B TurboQuant** | `start-prime-turboquant-256k.sh` | After GGUF/fork compatibility fixed |
| **Upgrade → cloud** | `/mode cloud` | Need frontier coding quality now |
| **Rollback → Qwen 35B MoE 128K** | `start-prime.sh` | Maximum MoE speed, 128K only |
| **Baseline Gemma (no spec)** | `PRIME_SPEC_TYPE=none` | Debug speculation issues |

Qwen3.6-27B weights already on disk: `~/.cache/huggingface/llamacpp-qwen36-27b/Qwen3.6-27B-Q4_K_M.gguf` (17.1 GB).

---

## 11. Known gotchas

1. **QAT ≠ TurboQuant.** QAT is Unsloth weight quantization in the GGUF. TurboQuant is a **KV cache compression fork** (`llama-cpp-turboquant`). Do **not** combine TurboQuant KV with Gemma MTP — community reports ~50% throughput loss.  
2. **MTP assistant is mandatory** for champion profile. Without `gemma-4-26B-A4B-it-assistant-Q2_K.gguf`, launch fails or use `PRIME_SPEC_TYPE=none`.  
3. **Chat template required.** `--jinja` + `google-gemma-4-31B-it-interleaved.jinja` — missing file = exit 1.  
4. **Two llama.cpp trees.** You run **stock** `~/Projects/llama.cpp`. TurboQuant lives in `~/Projects/llama-cpp-turboquant` for future dense 27B only.  
5. **Config drift.** Clients may still say `qwen3.6-27b` — server id is `gemma-4-26b-a4b-it`.  
6. **Reasoning mode off.** `--reasoning off` on server; do not expect `` blocks unless client enables thinking compat.  
7. **Port guard in launch script.** If `:8000` already responds, script sleeps forever (systemd-friendly no-op). Stop old Prime before manual cutover.

---

## 12. Parked experiments

| Experiment | Status |
|------------|--------|
| Qwen3.6-27B TurboQuant | Weights OK; **load failed** on turboquant fork |
| Qwen3-Coder-Next | Aborted (46 GB) |
| DiffusionGemma | **Evaluated 2026-06-11 — NO-GO** (23 tok/s vs 186 tok/s Prime); see `docs/PRIME_HANDOFF_DIFFUSIONGEMMA_26B.md` |
| Sovereign MoE `:8010` | Broken |

---

## 13. One-paragraph self-summary (inject into system context)

I am **Prime**, `gemma-4-26b-a4b-it`, a Gemma 4 **26B-A4B MoE** (~3.8B active params) running on dual RTX 5070 Ti via **stock llama.cpp** at `http://localhost:8000/v1` with **262144** context. I use **QAT UD-Q4_K_XL** weights (~14 GB), **f16 KV**, and the sweep **champion** speculative stack (`draft-mtp` + `ngram-mod`, assistant Q2_K). I was chosen because this is the **only profile that reliably delivers true 256K on 32 GB VRAM** at **~185+ tok/s** without RAM spill; denser models (Qwen3.6-27B) are the planned upgrade but failed to load on the TurboQuant fork. Trade-off: **fast and long-context**, weaker at hard coding than dense 27B. I serve Pi (interactive agent) and GZMO (ingest, dream, memory, distill). Embeddings/rerank: VM200 `:8081`; persistence: LXC101 (Qdrant, Redis, Neo4j). Config: `gzmo.toml`. Launch: `start-prime-gemma4-26b-a4b-256k.sh`.

---

## 14. Operator checklist — seed this handoff

- [ ] Align `gzmo.toml` + Pi `models.json` to `gemma-4-26b-a4b-it`  
- [ ] `gzmo ingest docs/PRIME_HANDOFF_GEMMA4_26B.md` (vault recall)  
- [ ] Append §13 to Pi tiered-memory or `MEMORY_CORE.md`  
- [ ] Update `docs/PORTS.md` Prime table if still listing wrong model  
- [ ] Re-run `scripts/seed-core-stack.py` + `gzmo memory embed` after Prime card edits  

---

*Generated for operator handoff to production Gemma 4 26B-A4B Prime, 2026-06-11.*
