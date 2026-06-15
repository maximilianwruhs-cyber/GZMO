# GZMO — Core Stack Knowledge (curated, self-describing)

**Status:** 2026-06-07 (clean-slate seed)
**Repo:** `/home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO`
**Role:** The canonical, hand-curated knowledge base of GZMO's own machine — what every stack entity is, how it works, how to use it, and why it was built that way. This document is both (a) the human source of truth and (b) the **template** for how all core knowledge entries should look. The dense `[TYPE:Name]` lines under each card are injected directly into `vault.db` (vault + honeypot) by [`scripts/seed-core-stack.py`](../scripts/seed-core-stack.py).

> This is curated, operator-authored self-knowledge — not migration-pile data. It is the first and lowest-risk thing populated into a clean system.

---

## How to read / extend this document (the template)

Every entity is one card:

```
## [TYPE:Name]
- What:    canonical definition (the thing itself)
- How:     how it actually works / internals
- Use:     commands, when to touch it, what reads or writes it
- Why:     rationale, tradeoffs, what it replaced or rejected
- Related: [OtherEntity], [OtherEntity]
- Injected facts:
  - [TYPE:Name] one dense, self-contained statement (carries the "why" where it matters).
```

**Injection conventions** (match [`scripts/seed-cognition-stack.py`](../scripts/seed-cognition-stack.py)):
`decay_class = Structural`, `confidence = 0.95`, `origin = manual`, `container = obolus`, `source_file = manual/core_stack_20260607.md`. After seeding: `gzmo memory embed` then `scripts/sync-vault-to-qdrant.py --source honeypot`.

**Type tags:** `NODE` (hardware), `SERVICE` (running port), `MODEL` (weights), `CONFIG` (gzmo.toml), `CONCEPT` (design idea), `PROC` (process/pipeline), `SYSTEM` (engine/subsystem), `PATH` (storage), `HOWTO` (operator action), `POLICY` (doctrine), `STATE` (current status).

---

## Domain 1 — Nodes / topology

### [NODE:Workstation]
- What: The primary local machine; runs cognition (Prime), the gzmo daemon/CLI, the SQLite source of truth, the Pi frontend, and the Pi embed server.
- How: Ryzen 9950X CPU + 2x RTX 5070 Ti (16 GB each). No NVLink, so Prime is layer-split across both GPUs over PCIe.
- Use: All `gzmo` commands run here from the repo root `~/Projects/_foundation-audit/survey_GZMO`.
- Why: Local-first sovereignty — all heavy inference and the authoritative vault live on hardware the operator controls; nothing leaves the perimeter by default.
- Related: [SERVICE:Prime], [NODE:VM200], [NODE:LXC101]
- Injected facts:
  - [NODE:Workstation] The workstation (Ryzen 9950X, dual RTX 5070 Ti, no NVLink) is the GZMO cognition + source-of-truth node: it runs Prime, the gzmo daemon/CLI, SQLite vault.db, and the Pi frontend.
  - [NODE:Workstation] Because there is no NVLink, Prime is layer-split across both RTX 5070 Ti GPUs over PCIe (`-sm layer -dev CUDA0,CUDA1`).

### [NODE:VM200]
- What: The retrieval-model node (`ollamagpu`) hosting embeddings and rerank.
- How: VM at `192.168.31.110` with a GTX 1070 8 GB eGPU; serves a single unified retrieval router on `:8081` (`gzmo-embed` + `gzmo-rerank` presets).
- Use: Reached over HTTP from the workstation at `:8081` for both embed and rerank. Deployed via `scripts/vm200/deploy-retrieval-router.sh`. Librarian distill moved to Prime `:8000`.
- Why: Offloads light, frequent retrieval work onto cheap older GPU silicon so the workstation GPUs stay dedicated to Prime cognition.
- Related: [SERVICE:Embed], [SERVICE:Rerank]
- Injected facts:
  - [NODE:VM200] VM200 (ollamagpu, 192.168.31.110, GTX 1070 eGPU) is the retrieval layer: a unified llama-server router on :8081 serving embed + rerank — offloaded from the workstation so Prime keeps both 5070 Ti GPUs. Session distill uses Prime :8000 (VM200 :8082 rerank and :8083 librarian retired).

### [NODE:LXC101]
- What: The persistence node — Neo4j, Qdrant, and Redis.
- How: Docker host at `192.168.31.202` on the homelab Proxmox box.
- Use: Workstation connects over bolt `:7687` (Neo4j via MCP), HTTP `:6333` (Qdrant), `:6379` (Redis).
- Why: Keeps stateful databases off the workstation so reboots/rebuilds of cognition do not risk the graph or vector stores.
- Related: [SERVICE:Neo4j], [SERVICE:Qdrant], [SERVICE:Redis], [NODE:PVE]
- Injected facts:
  - [NODE:LXC101] LXC101 (192.168.31.202, Docker) is the persistence plane: Neo4j :7687, Qdrant :6333, Redis :6379 — kept off the workstation so cognition rebuilds never threaten the databases.

### [NODE:PVE]
- What: The Proxmox hypervisor hosting VM200 and the LXC containers.
- How: `192.168.31.200`, i7-6770HQ.
- Use: Infrastructure host; not on the GZMO hot path directly.
- Why: Consolidates homelab virtualization on one low-power box.
- Related: [NODE:VM200], [NODE:LXC101], [NODE:LXC100], [NODE:LXC102]
- Injected facts:
  - [NODE:PVE] PVE (192.168.31.200, i7-6770HQ) is the Proxmox hypervisor for VM200 and the LXC containers; it is infrastructure, not on the GZMO hot path.

### [NODE:LXC100]
- What: Samba file share container.
- How: `192.168.31.201`.
- Use: File sharing; not on the GZMO hot path.
- Why: General homelab storage, separate from GZMO stores.
- Related: [NODE:PVE]
- Injected facts:
  - [NODE:LXC100] LXC100 (192.168.31.201) runs Samba and is not part of the GZMO memory hot path.

### [NODE:LXC102]
- What: Optional MCP hub container (Pi-era).
- How: `192.168.31.203`.
- Use: Optional MCP hosting; not required for the current stack.
- Why: Reserved for distributing MCP servers; currently optional.
- Related: [SYSTEM:MCP]
- Injected facts:
  - [NODE:LXC102] LXC102 (192.168.31.203) is an optional MCP hub from the Pi era and is not required by the current stack.

---

## Domain 2 — Services / ports

### [SERVICE:Prime]
- What: Production cognition endpoint at `http://localhost:8000/v1` on the workstation.
- How: Stock llama.cpp `llama-server` serving **Gemma 4 26B-A4B-it** MoE QAT (alias `gemma-4-26b-a4b-it`, ctx **262144**), dual-GPU layer-split, champion profile **draft-mtp+ngram-mod** (assistant Q2_K), f16 KV, CUDA graphs off (`GGML_CUDA_DISABLE_GRAPHS=1`). Serves chat, ingest extract+verify, dream, spark, and session distill.
- Use: Start `~/Projects/llama.cpp/prime-bench/start-prime-gemma4-26b-a4b-256k.sh` or systemd `gzmo-prime.service`; health `curl http://localhost:8000/v1/models` → HTTP 200; configured in `gzmo.toml [engine.local]`.
- Why: **Locked over Qwen3.6-35B:** same dual-5070 Ti rig runs **256K ctx** vs Qwen's 128K — more headroom for long ingest chunks, dream REM, and archive-aware chat without cloud.
- Related: [MODEL:Prime], [CONFIG:engine.local], [NODE:Workstation]
- Injected facts:
  - [SERVICE:Prime] Production cognition runs at http://localhost:8000/v1 via stock llama.cpp llama-server with draft-mtp+ngram-mod champion profile; it is the heavy-inference endpoint for chat, ingest extract/verify, dream, spark, and session distill.
  - [SERVICE:Prime] Start Prime with start-prime-gemma4-26b-a4b-256k.sh (or systemd gzmo-prime.service); confirm health with curl http://localhost:8000/v1/models returning HTTP 200.
  - [SERVICE:Prime] Prime is Gemma 4 26B-A4B MoE at ctx 262144 — chosen over Qwen3.6-35B because the same hardware achieves double the context window (256K vs 128K).

### [SERVICE:Embed]
- What: GZMO vault/honeypot embedding endpoint.
- How: VM200 `http://192.168.31.110:8081/v1`, model `Qwen3-Embedding-0.6B-Q8_0.gguf`, 1024-dim vectors, `cache_ttl_secs = 86400`.
- Use: Used by `gzmo memory embed`, recall similarity, and Qdrant sync. Config `gzmo.toml [embeddings]`.
- Why: Small fast embedder on cheap GPU; 1024-dim matches the Qdrant collections. Distinct from the Pi embed on `:8002`.
- Related: [SERVICE:PiEmbed], [SERVICE:Qdrant], [CONFIG:embeddings]
- Injected facts:
  - [SERVICE:Embed] GZMO vault/honeypot embeddings come from VM200 :8081 (Qwen3-Embedding-0.6B-Q8, 1024-dim, cache_ttl 86400s), configured in gzmo.toml [embeddings].
  - [SERVICE:Embed] The GZMO embed endpoint (:8081) is separate from the Pi knowledge-base embed on workstation :8002.

### [SERVICE:Rerank]
- What: Recall post-filter reranker.
- How: VM200 retrieval router `http://192.168.31.110:8081/v1`, model `gzmo-rerank` (Qwen3-Reranker-0.6B), `prefetch_multiplier = 4`.
- Use: Applied after RRF fusion in `memory_search`; config `gzmo.toml [rerank]`.
- Why: Cross-encoder reranking sharply improves top-k precision; prefetch 4x gives it candidates to reorder.
- Related: [PROC:Recall], [CONFIG:rerank]
- Injected facts:
  - [SERVICE:Rerank] Recall is post-filtered by the VM200 reranker preset gzmo-rerank on the :8081 router (Qwen3-Reranker-0.6B, prefetch_multiplier 4) after RRF fusion, per gzmo.toml [rerank]. The legacy standalone :8082 bge reranker is retired.

### [SERVICE:Librarian]
- What: Session distill extract/summary routing profile (points at Prime).
- How: `http://localhost:8000/v1`, model `gemma-4-26b-a4b-it`, temp 0.2, max_tokens 4096 via `LibrarianConfig::to_engine_profile`.
- Use: SessionDistill extract + summary; config `gzmo.toml [routing.mappings]` (`distill_* = local`). `[librarian].enabled = false`, `use_librarian = false`.
- Why: MoE A4B (~4B active params/token) handles distill bulk work on Prime; VM200 :8083 Qwen 1.5B retired.
- Related: [SYSTEM:SessionDistill], [CONFIG:librarian], [SERVICE:Prime]
- Injected facts:
  - [SERVICE:Librarian] Session distill extract/summary routes to Prime :8000 (gemma-4-26b-a4b-it, temp 0.2) via gzmo.toml [librarian]; VM200 :8083 is deprecated.

### [SERVICE:PiEmbed]
- What: Pi knowledge-base embedding endpoint (and GZMO fallback).
- How: Workstation `:8002`, started by `scripts/start-embed.sh` or `gzmo-embed.service`.
- Use: Pi `knowledge_search` indexing; config `~/.pi/agent/knowledge-base.json`.
- Why: Local embed for Pi raw-document RAG, independent of the VM200 GZMO embedder.
- Related: [SERVICE:Embed], [CONCEPT:Pi], [SERVICE:Qdrant]
- Injected facts:
  - [SERVICE:PiEmbed] Workstation :8002 is the Pi knowledge-base embed server (start-embed.sh / gzmo-embed.service); Pi raw-doc indexing uses it, not the VM200 GZMO embedder.

### [SERVICE:Neo4j]
- What: Knowledge-graph store for entities and relations with provenance.
- How: LXC101 bolt `:7687`, reached only via the `mcp-neo4j-memory` MCP server (stdio), tools `mcp__memory__*`.
- Use: Written by ingest, dream, and spark; credentials in `.env`.
- Why: Explicit graph complements vector recall; a stream in RRF. MCP-mediated so writes are uniform across engines.
- Related: [SYSTEM:MCP], [SERVICE:Qdrant], [PROC:Ingest]
- Injected facts:
  - [SERVICE:Neo4j] Neo4j (LXC101 bolt :7687) holds the entity/relation graph and is accessed only through the mcp-neo4j-memory MCP server (mcp__memory__* tools), written by ingest, dream, and spark.

### [SERVICE:Qdrant]
- What: Vector store for RAG.
- How: LXC101 `http://192.168.31.202:6333`. Collections: `honeypot` (production RAG, mirrors SQLite honeypot `is_latest=1`), `knowledge` (Pi raw-doc index), `knowledge_core` (M5 ripened cards). 1024-dim cosine.
- Use: Synced nightly 01:45 UTC by the daemon, or manually `scripts/sync-vault-to-qdrant.sh`.
- Why: SQLite stays source of truth; Qdrant is the shared association field. Only honeypot (curated) is mirrored — never the whole vault.
- Related: [CONCEPT:FourLayers], [HOWTO:SyncQdrant], [SERVICE:Embed]
- Injected facts:
  - [SERVICE:Qdrant] Qdrant (LXC101 :6333) holds collections honeypot (production RAG), knowledge (Pi raw docs), and knowledge_core (M5); honeypot mirrors only the curated SQLite honeypot rows, never the full vault.
  - [SERVICE:Qdrant] Qdrant honeypot is synced from SQLite nightly at 01:45 UTC or manually via scripts/sync-vault-to-qdrant.sh; SQLite vault.db remains the source of truth.

### [SERVICE:Redis]
- What: Hot scratch cache and the distill work queue.
- How: LXC101 `redis://192.168.31.202:6379`; queue key `gzmo:distill:pending`.
- Use: Per-turn scratch and the BRPOP distill worker; config `gzmo.toml [redis]`.
- Why: Fast ephemeral memory for the current turn and decoupled async distillation of archived context.
- Related: [CONCEPT:HotMemory], [SYSTEM:DistillWorker], [CONFIG:context_memory]
- Injected facts:
  - [SERVICE:Redis] Redis (LXC101 :6379) provides per-turn scratch memory and the distill queue gzmo:distill:pending consumed by the daemon's BRPOP distill worker.

### [SERVICE:Parked]
- What: Intentionally-down endpoints.
- How: Sovereign FrankenMoE `:8010` (broken MoE GGUF), VM200 brain `:8080` (retired 7B).
- Use: Do not start; do not block on them.
- Why: Documented so they are not mistaken for live services; kept down until a working build exists.
- Related: [SERVICE:Prime], [MODEL:Prime]
- Injected facts:
  - [SERVICE:Parked] Sovereign FrankenMoE :8010 and VM200 brain :8080 are intentionally down (broken MoE output / retired 7B); they are not part of the live stack and must not be blocked on.

---

## Domain 3 — Models

### [MODEL:Prime]
- What: The production cognition model.
- How: **Gemma 4 26B-A4B-it** MoE QAT (UD-Q4_K_XL), alias `gemma-4-26b-a4b-it`, served by stock llama.cpp at ctx **262144**.
- Use: `gzmo.toml [engine.local] model`. Temperature 0.3 for chat; 0.1 deterministic profile for ingest extract.
- Why: **256K context on dual 16 GB GPUs** — primary reason for choosing Gemma over Qwen3.6-35B (128K max on same rig). MoE A4B keeps active compute manageable.
- Related: [SERVICE:Prime], [CONFIG:engine.local]
- Injected facts:
  - [MODEL:Prime] The Prime model is Gemma 4 26B-A4B-it MoE QAT (alias gemma-4-26b-a4b-it) at ctx 262144 — locked over Qwen3.6-35B for double the context window on the same dual-5070 Ti hardware.

### [MODEL:Embed]
- What: Embedding model for GZMO memory.
- How: `Qwen3-Embedding-0.6B-Q8_0.gguf`, 1024-dim.
- Use: VM200 :8081; vault/honeypot vectors.
- Why: Small, fast, 1024-dim to match Qdrant collections.
- Related: [SERVICE:Embed]
- Injected facts:
  - [MODEL:Embed] GZMO uses Qwen3-Embedding-0.6B-Q8 (1024-dim) on VM200 :8081 for all vault/honeypot embeddings.

### [MODEL:Rerank]
- What: Cross-encoder reranker.
- How: `Qwen3-Reranker-0.6B` (`gzmo-rerank` preset on the VM200 :8081 router).
- Use: VM200 :8081; recall post-filter.
- Why: Unifies embed + rerank on one llama-server; shares the Qwen3 retrieval stack.
- Related: [SERVICE:Rerank]
- Injected facts:
  - [MODEL:Rerank] The reranker is Qwen3-Reranker-0.6B (gzmo-rerank preset) on the VM200 :8081 router, used to post-filter recall candidates. The former bge-reranker-v2-m3 on :8082 is retired.

### [MODEL:Librarian]
- What: Session distill routing alias (same weights as Prime).
- How: `gemma-4-26b-a4b-it` on Prime `:8000`; librarian profile uses lower temp (0.2) for extract/summary.
- Use: Session distill extract + summary via `[librarian]` shortcut.
- Why: Unified stack on champion Prime; MoE keeps per-token cost manageable for off-peak distill cron.
- Related: [SERVICE:Librarian], [MODEL:Prime]
- Injected facts:
  - [MODEL:Librarian] The librarian profile uses Prime gemma-4-26b-a4b-it at :8000 (temp 0.2) for session-distill extract and summary.

### [MODEL:Cloud]
- What: Optional cloud fallback cognition.
- How: OpenRouter `nvidia/nemotron-3-super-120b-a12b:free`; fallback `gemini-2.5-flash`. Activated via `/mode cloud`.
- Use: `gzmo.toml [engine.cloud]`; keys in `.env` (`GZMO_OPENROUTER_KEY`, `GZMO_GEMINI_KEY`).
- Why: Optional burst capacity; off by default to preserve sovereignty.
- Related: [CONFIG:engine.local], [SERVICE:Prime]
- Injected facts:
  - [MODEL:Cloud] Optional cloud mode (/mode cloud) uses OpenRouter nemotron-3-super-120b (free) with gemini-2.5-flash fallback; it is off by default to keep GZMO local-first.

### [MODEL:TurboQuant]
- What: A llama.cpp fork for long-context KV quantization.
- How: `~/Projects/llama-cpp-turboquant` (turbo2/turbo3/turbo4) enabling ~256K context on ~32 GB VRAM within ~5% perplexity of q8_0.
- Use: Gated by `scripts/turbo-quality-gate.sh`; basis for the Gemma 4 cutover.
- Why: Enables much longer context locally without an exotic GPU; quality-gated before adoption.
- Related: [MODEL:Gemma4Cutover], [SERVICE:Prime]
- Injected facts:
  - [MODEL:TurboQuant] TurboQuant (~/Projects/llama-cpp-turboquant) is a llama.cpp fork giving ~256K context on ~32 GB VRAM within ~5% perplexity of q8_0, gated by turbo-quality-gate.sh.

### [MODEL:Gemma4Cutover]
- What: **Completed** — Gemma 4 26B-A4B MoE is now Prime (not a future cutover).
- How: `gemma-4-26B-A4B-it-qat-UD-Q4_K_XL.gguf` @ 262144 via stock llama.cpp; requires `google-gemma-4-31B-it-interleaved.jinja` with `--jinja`.
- Use: `start-prime-gemma4-26b-a4b-256k.sh`; retired Qwen path is `start-prime.sh` (128K rollback only).
- Why: 256K ctx on dual 5070 Ti without TurboQuant fork; champion profile draft-mtp+ngram-mod, f16 KV, CUDA graphs off.
- Related: [MODEL:Prime], [SERVICE:Prime], [POLICY:NoBulkSwap]
- Injected facts:
  - [MODEL:Gemma4Cutover] Prime cutover to Gemma 4 26B-A4B champion (draft-mtp+ngram-mod @ 256K) is complete; Qwen3.6-35B @ 128K is retired to start-prime.sh for rollback only.
  - [MODEL:Gemma4Cutover] Gemma 4 instruct requires google-gemma-4-31B-it-interleaved.jinja with --jinja; the legacy --chat-template gemma (Gemma 3) causes repetition/gibberish.

---

## Domain 4 — Config spine (`gzmo.toml`)

### [CONFIG:gzmo_toml]
- What: The single runtime authority for the whole stack.
- How: Every client and the daemon read `gzmo.toml`; secrets come from `.env`.
- Use: Path override `GZMO_CONFIG`; do not commit secrets.
- Why: One source of truth prevents config drift across CLI, daemon, and Pi bridge.
- Related: [CONFIG:engine.local], [CONFIG:ingest], [CONFIG:qdrant]
- Injected facts:
  - [CONFIG:gzmo_toml] gzmo.toml is the single runtime config authority read by the CLI, daemon, and Pi bridge; secrets live in .env and are never committed.

### [CONFIG:engine.local]
- What: Local Prime engine profile.
- How: `url = http://localhost:8000/v1`, `model = gemma-4-26b-a4b-it`, `temperature = 0.3`, `top_p = 0.95`, `max_tokens = 24576`; ingest extract uses the `local_deterministic` profile at `temperature = 0.1`.
- Use: Active mode set by `[engine] active_mode = "local"`; switch with `/mode`.
- Why: 24k output cap is a guardrail so a runaway generation fails fast rather than consuming the full 256K context.
- Related: [SERVICE:Prime], [MODEL:Prime], [CONFIG:ingest]
- Injected facts:
  - [CONFIG:engine.local] gzmo.toml [engine.local] points Prime at http://localhost:8000/v1 (gemma-4-26b-a4b-it, temp 0.3, max_tokens 24576); ingest extraction uses a deterministic temp 0.1 profile.
  - [CONFIG:engine.local] The 24576 max_tokens cap is a deliberate guardrail so a looping generation fails fast instead of consuming the full 262144 context.

### [CONFIG:ingest]
- What: IngestEngine gates.
- How: `enabled = true`, `max_source_chars = 120000`, `chunk_chars = 28000`, `verify = true`, `min_confidence = 0.85`, `require_evidence = true`, `strict_kg = true`.
- Use: Governs `gzmo ingest` and the (currently disabled) watcher.
- Why: Hard gates enforce the moat: nothing enters memory without verified, evidence-backed, high-confidence facts.
- Related: [PROC:Ingest], [CONCEPT:Honeypot], [POLICY:CurationFirst]
- Injected facts:
  - [CONFIG:ingest] gzmo.toml [ingest] enforces verify=true, min_confidence=0.85, require_evidence=true, strict_kg=true, max_source_chars=120000, chunk_chars=28000 — the gates that keep unverified facts out of memory.

### [CONFIG:qdrant]
- What: Qdrant sync config.
- How: `url = http://192.168.31.202:6333`, `collection = "honeypot"`, `sync_enabled = true`, `sync_cron_hour = 1`, `sync_cron_minute = 45`.
- Use: Daemon nightly sync; manual `sync-vault-to-qdrant.sh`.
- Why: Mirrors only honeypot; nightly cadence balances freshness vs load (the F15 same-night gap is a known tradeoff).
- Related: [SERVICE:Qdrant], [HOWTO:SyncQdrant]
- Injected facts:
  - [CONFIG:qdrant] gzmo.toml [qdrant] syncs the honeypot collection to LXC101 :6333 nightly at 01:45 UTC; same-night writes after 01:45 miss Qdrant until the next sync (F15) — run sync-vault-to-qdrant.sh to close the gap.

### [CONFIG:redis]
- What: Redis + distill queue config.
- How: `url = redis://192.168.31.202:6379`, `distill_queue = gzmo:distill:pending`.
- Use: Scratch + distill worker.
- Why: Decouples archive-time context pruning from distillation.
- Related: [SERVICE:Redis], [SYSTEM:DistillWorker]
- Injected facts:
  - [CONFIG:redis] gzmo.toml [redis] sets redis://192.168.31.202:6379 with distill queue gzmo:distill:pending for scratch memory and async distillation.

### [CONFIG:context_memory]
- What: Hot-context budget config.
- How: `archive_threshold = 0.90`, `scratch_max_tokens = 2000`, `context_length = 262144`.
- Use: Drives context prune → archive → distill at 90% budget.
- Why: Prevents context overflow while capturing pruned content into the distill pipeline.
- Related: [SERVICE:Redis], [SYSTEM:DistillWorker], [PROC:Recall]
- Injected facts:
  - [CONFIG:context_memory] gzmo.toml [context_memory] prunes hot context at 90% of the 262144 budget (scratch_max_tokens 2000), enqueueing archived content for distillation.

### [CONFIG:subagent]
- What: Subagent concurrency.
- How: `max_concurrent = 2`.
- Use: Caps parallel subagents.
- Why: Bounds resource use on the shared workstation GPUs.
- Related: [SERVICE:Prime]
- Injected facts:
  - [CONFIG:subagent] gzmo.toml [subagent] caps max_concurrent at 2 to bound load on the shared workstation GPUs.

### [CONFIG:platform_search]
- What: Cross-search config (honeypot + Pi knowledge).
- How: `include_knowledge_collection = true`, `knowledge_collection = "knowledge"`, `knowledge_prefetch = 12`.
- Use: `gzmo_memory_search` merges honeypot recall with the Pi `knowledge` Qdrant collection (read-only).
- Why: Lets GZMO recall both its curated facts and Pi's raw-doc index in one query.
- Related: [SYSTEM:MCP], [CONCEPT:Pi], [SERVICE:Qdrant]
- Injected facts:
  - [CONFIG:platform_search] gzmo.toml [platform_search] cross-searches the Pi knowledge Qdrant collection (read-only, prefetch 12) alongside honeypot recall in gzmo_memory_search.

---

## Domain 5 — Memory model

### [CONCEPT:FourLayers]
- What: GZMO's memory is four explicit layers, not one vector store.
- How: Document → Vault (ops, all verified facts) → Honeypot (Tier-1 curated) → Mature core (M5 ripened). Evidence (Tier-2 spans) backs honeypot; Qdrant mirrors honeypot only.
- Use: Recall fuses these streams; ingest promotes through them.
- Why: Separates "this is RAG" (honeypot) from "this is ops history" (vault), the core design improvement over a vault-soup-with-Qdrant-mirror.
- Related: [CONCEPT:Honeypot], [CONCEPT:Evidence], [SERVICE:Qdrant], [PATH:knowledge_core_db]
- Injected facts:
  - [CONCEPT:FourLayers] GZMO memory has four layers: vault (all verified facts, ops soup), honeypot (Tier-1 curated crystal), evidence (Tier-2 source spans), and knowledge_core (M5 ripened); Qdrant mirrors only honeypot.

### [CONCEPT:Honeypot]
- What: The curated Tier-1 recall field.
- How: SQLite `honeypot` table + `honeypot_fts`; a row qualifies when confidence ≥ 0.85, `source_file` is non-empty, it is not a `[relation:...]` row, and the source path is not under excluded patterns (`Sources/`, `Chat_History/`, `Quelltext/`).
- Use: Primary RAG; the source for the Qdrant honeypot collection; Dream/Spark anchors.
- Why: A high-trust subset (~quarter of vault historically) keeps recall precise and the association field clean.
- Related: [CONCEPT:FourLayers], [CONCEPT:Evidence], [SERVICE:Qdrant]
- Injected facts:
  - [CONCEPT:Honeypot] The honeypot is GZMO's curated Tier-1 recall layer; a fact qualifies only with confidence >=0.85, a non-empty source_file, not being a [relation:] row, and a source path outside Sources/Chat_History/Quelltext.
  - [CONCEPT:Honeypot] Honeypot is the default for recall, the Qdrant mirror source, and Dream/Spark anchors — the curated crystal, not the full vault soup.

### [CONCEPT:Evidence]
- What: Tier-2 source spans grounding each honeypot fact.
- How: SQLite `evidence` table + `evidence_fts`, 1:1 with `honeypot.id`, storing char start/end and the quoted text, plus a local embedding stream.
- Use: Strict recall grounding; `source_span:` in scratch.
- Why: Verifiable provenance — every curated fact can point at the exact text that justified it. Empty evidence collapses strict recall.
- Related: [CONCEPT:Honeypot], [PROC:Ingest]
- Injected facts:
  - [CONCEPT:Evidence] The evidence table (Tier-2) stores quotable source spans 1:1 with honeypot facts (char offsets + text + local vectors); it is the verifiable provenance behind strict recall.

### [CONCEPT:Episodic]
- What: The raw daily log.
- How: `memory/YYYY-MM-DD.md`; ingest writes `[ingest:...]` receipts; the DreamEngine consolidates yesterday's log.
- Use: Dream substrate and provenance, not a primary recall store.
- Why: Captures the day's stream cheaply; dreams distill it into vault truths.
- Related: [SYSTEM:Dream], [PATH:memory_dir]
- Injected facts:
  - [CONCEPT:Episodic] Episodic logs (memory/YYYY-MM-DD.md) are the raw daily stream and the dream substrate; they hold ingest receipts but are not a primary recall store.

### [CONCEPT:Wiki]
- What: The emit-only markdown synthesis layer.
- How: `wiki/` git-tracked pages; WikiEngine emits on ingest (`emit_on_ingest = true`) and runs sync/lint loops; retrieval is grep via `gzmo wiki search` / `gzmo_wiki_search`.
- Use: Browsable synthesis between raw RAG and DREAMS.md; never re-ingested.
- Why: Human-browsable knowledge that compounds without polluting the verified stores.
- Related: [SYSTEM:WikiEngine], [PATH:wiki_dir]
- Injected facts:
  - [CONCEPT:Wiki] The wiki/ layer is emit-only: WikiEngine writes pages on ingest and is searched by grep (gzmo wiki search), but wiki pages are never re-ingested into vault.

---

## Domain 6 — Ingest pipeline

### [PROC:Ingest]
- What: The distillation pipeline that turns a document into verified memory.
- How: `ingest_prep` (strip frontmatter, classify) → Prime extract (:8000, temp 0.1) → verify-on-merged → `promote_truths` → `semantic_vault` → honeypot (if qualifies) + evidence (Tier-2 localize) → Neo4j MCP → episodic receipt → optional wiki emit → Qdrant sync.
- Use: `gzmo ingest <file>`; dry-run contract via `gzmo ingest-eval` (no writes).
- Why: GZMO is a distillation pipeline, not a chatbot with a vector store; the LLM thinks (extract/verify) and the pipeline remembers (vault/honeypot/evidence/graph).
- Related: [CONFIG:ingest], [CONCEPT:Honeypot], [CONCEPT:Evidence], [SERVICE:Neo4j]
- Injected facts:
  - [PROC:Ingest] Ingest flows: ingest_prep -> Prime extract (:8000, temp 0.1) -> verify-on-merged -> promote -> semantic_vault -> honeypot+evidence if it qualifies -> Neo4j MCP -> episodic receipt -> optional wiki emit -> Qdrant sync.
  - [PROC:Ingest] gzmo ingest-eval is a dry-run contract check that writes nothing to vault/honeypot/evidence/Neo4j; only gzmo ingest (live) writes memory.

### [PROC:Recall]
- What: The hybrid recall pipeline.
- How: `recall_rrf` fuses honeypot FTS, evidence FTS, graph/keyword, vector (Qdrant + local), and evidence-vector streams via Reciprocal Rank Fusion, then reranks on the VM200 :8081 router and diversifies by source_file.
- Use: `gzmo memory search`, `gzmo_memory_search` MCP, Pi bridge.
- Why: No single retrieval method is enough; RRF + rerank balances lexical, semantic, and graph signals.
- Related: [SERVICE:Rerank], [CONCEPT:Honeypot], [SERVICE:Qdrant]
- Injected facts:
  - [PROC:Recall] Recall (recall_rrf) fuses honeypot FTS, evidence FTS, graph/keyword, and vector streams with RRF, then reranks on the VM200 :8081 router and diversifies by source_file.

---

## Domain 7 — Cognition engines + schedule

> As of the 2026-06-07 clean slate, Dream, Spark, and SessionDistill are set `enabled = false` in `gzmo.toml`. The cards below describe their design and cron so the system understands them; re-enable after curated population.

### [SYSTEM:Dream]
- What: Nightly consolidation of episodic logs into vault truths.
- How: DreamEngine at 01:00 UTC; reads yesterday's `memory/*.md`, filters janitor/spark/ingest echoes, REM-chunks (28000 chars), verifies (min_confidence 0.85), writes vault + honeypot + Neo4j; uses honeypot anchors (`honeypot_rem_enabled`).
- Use: `gzmo dream` manual; cron in `[dreams]`.
- Why: Compresses raw days into durable semantic truths; verifier blocks hallucinated dreams from becoming wisdom.
- Related: [CONCEPT:Episodic], [SYSTEM:Spark], [PATH:dreams_md]
- Injected facts:
  - [SYSTEM:Dream] DreamEngine ([dreams], 01:00 UTC) consolidates yesterday's episodic log into vault+honeypot+Neo4j with verify at min_confidence 0.85; run manually with gzmo dream. Currently disabled during clean-slate rebuild.

### [SYSTEM:SessionDistill]
- What: Turns chat sessions into vault facts.
- How: 02:15 UTC; reads `data/sessions/*.json`, extracts + summarizes on Prime (librarian profile), fact-checks on Prime, promotes to vault + honeypot.
- Use: `[session_distill]`; also runs via the Redis distill worker on archived context.
- Why: Captures durable facts from conversations without manual curation of every turn.
- Related: [SERVICE:Librarian], [SYSTEM:DistillWorker], [PATH:sessions_dir]
- Injected facts:
  - [SYSTEM:SessionDistill] SessionDistill ([session_distill], 02:15 UTC) distills data/sessions/*.json into vault+honeypot via Prime librarian-profile extract/summary + Prime fact-check.

### [SYSTEM:Spark]
- What: Serendipitous recall connecting old and recent memory.
- How: 03:30 and 22:30 UTC; samples honeypot anchors (`anchor_decay_classes = ["CuratedVault","SessionDistill"]`), hypothesizes a connection, verifies, and promotes Neo4j relations only (HYPOTHESIZED_LINK) at quarantine confidence 0.6.
- Use: `gzmo spark`; cron in `[spark]`.
- Why: Generates novel links without polluting honeypot facts; the "dice that revisits something old."
- Related: [SYSTEM:Dream], [SERVICE:Neo4j], [CONCEPT:Honeypot]
- Injected facts:
  - [SYSTEM:Spark] SparkEngine ([spark], 03:30/22:30 UTC) samples honeypot anchors, hypothesizes a connection, verifies, and writes only Neo4j HYPOTHESIZED_LINK relations (never honeypot facts) at quarantine confidence 0.6. Currently disabled during clean-slate rebuild.

### [SYSTEM:Janitor]
- What: Orchestrator maintenance loop.
- How: `sys_janitor` every 30 minutes; can write vault via tools.
- Use: Background maintenance.
- Why: Keeps the orchestrator healthy between heavy engine runs.
- Related: [SYSTEM:Dream]
- Injected facts:
  - [SYSTEM:Janitor] sys_janitor runs every 30 minutes as orchestrator maintenance and can write vault via tools.

### [SYSTEM:SynapsePull]
- What: Pi event tail into episodic.
- How: 02:45 UTC; reads `data/Synapse/events.jsonl` (append-only bus, never consumed for state) and appends to episodic.
- Use: `[synapse_pull]`, `max_events = 50`.
- Why: Surfaces Pi activity into GZMO's day log without coupling state.
- Related: [CONCEPT:Pi], [PATH:synapse_bus]
- Injected facts:
  - [SYSTEM:SynapsePull] Synapse pull ([synapse_pull], 02:45 UTC) tails the append-only Pi bus data/Synapse/events.jsonl into episodic; the bus is never consumed for state.

### [SYSTEM:KGReconcile]
- What: Neo4j ontology reconcile.
- How: 04:00 UTC, `dry_run = true` by default.
- Use: `[kg_reconcile]`.
- Why: Periodic graph hygiene; defaults to dry-run so it never silently rewrites the graph.
- Related: [SERVICE:Neo4j]
- Injected facts:
  - [SYSTEM:KGReconcile] KG reconcile ([kg_reconcile], 04:00 UTC) runs Neo4j ontology reconciliation in dry_run=true by default so it never silently rewrites the graph.

### [SYSTEM:WikiEngine]
- What: The wiki maintenance loops.
- How: Daily sync 05:30 UTC, weekly lint Sunday 06:00; emits pages on ingest.
- Use: `[wiki]`.
- Why: Keeps the synthesis layer indexed and structurally clean.
- Related: [CONCEPT:Wiki], [PATH:wiki_dir]
- Injected facts:
  - [SYSTEM:WikiEngine] WikiEngine ([wiki]) emits pages on ingest and runs daily index sync (05:30 UTC) plus weekly lint (Sunday 06:00 UTC).

### [SYSTEM:DistillWorker]
- What: Continuous async distiller.
- How: BRPOP on Redis `gzmo:distill:pending`; runs SessionDistill on archived context pruned at 90% budget.
- Use: Runs inside the daemon.
- Why: Decouples context pruning from the heavier distillation step.
- Related: [SERVICE:Redis], [SYSTEM:SessionDistill], [CONFIG:context_memory]
- Injected facts:
  - [SYSTEM:DistillWorker] The distill worker BRPOPs Redis gzmo:distill:pending and distills context archived at the 90% budget threshold, decoupling pruning from distillation.

---

## Domain 8 — MCP + integration

### [SYSTEM:MCP]
- What: The two MCP surfaces wiring Cursor, Pi, and GZMO to the stores.
- How: `memory` (stdio → `uvx mcp-neo4j-memory@0.4.5`) for Neo4j KG; `gzmo-memory` (stdio → `gzmo mcp-serve`) exposing `gzmo_memory_search`, `gzmo_memory_recall_pull`, `gzmo_memory_status`, `gzmo_wiki_search`. Installed by `scripts/install-shared-mcp.sh`.
- Use: Cursor + Pi read both; the daemon also spawns the Neo4j MCP as a client.
- Why: Uniform tool surface so every client touches memory the same way.
- Related: [SERVICE:Neo4j], [CONCEPT:Pi], [PROC:Recall]
- Injected facts:
  - [SYSTEM:MCP] Two MCP servers wire the stack: memory (uvx mcp-neo4j-memory@0.4.5) for the Neo4j graph, and gzmo-memory (gzmo mcp-serve) exposing gzmo_memory_search/recall_pull/status/wiki_search.
  - [SYSTEM:MCP] MCP configs are merged into ~/.cursor/mcp.json, ~/.pi/agent/mcp.json, and ~/.config/mcp/mcp.json by scripts/install-shared-mcp.sh.

### [CONCEPT:Pi]
- What: The operator frontend (pi-rust).
- How: Talks to Prime :8000 for cognition; reaches GZMO memory only via `scripts/pi-gzmo-memory.sh` (turn-start/search/recall); maintains its own raw-doc index in Qdrant `knowledge` via embed :8002.
- Use: `pi-gzmo-memory.sh prep "query"`; never touches Redis or vault SQL directly.
- Why: Clean separation — Pi is the frontend, GZMO is the memory platform.
- Related: [SYSTEM:MCP], [SERVICE:PiEmbed], [CONFIG:platform_search]
- Injected facts:
  - [CONCEPT:Pi] Pi (pi-rust) is the operator frontend; it uses Prime :8000 for cognition and reaches GZMO memory only through scripts/pi-gzmo-memory.sh, never Redis or vault SQL directly.
  - [CONCEPT:Pi] Pi maintains its own raw-document index in Qdrant collection "knowledge" via embed :8002, cross-searched read-only by GZMO platform_search.

---

## Domain 9 — Storage paths

### [PATH:vault_db]
- What: The SQLite source of truth.
- How: `data/vault.db` (schema v7, WAL): tables `semantic_vault`, `quarantine_vault`, `memory_index`, `honeypot`(+fts), `evidence`(+fts), `distill_dedup`, `ingest_dedup`.
- Use: All memory reads/writes; daemon holds the WAL (stop daemon before purge).
- Why: SQLite is durable, local, and the single authority that Qdrant/Neo4j mirror from.
- Related: [CONCEPT:FourLayers], [HOWTO:NuclearPurge]
- Injected facts:
  - [PATH:vault_db] data/vault.db (SQLite schema v7, WAL) is the source of truth holding semantic_vault, honeypot(+fts), evidence(+fts), quarantine_vault, and dedup tables; stop the daemon before touching it.

### [PATH:knowledge_core_db]
- What: The M5 ripened concept store.
- How: `data/knowledge_core.db`; cards built by `scripts/ripen-knowledge-core.py` under residency/corroboration gates.
- Use: Long-horizon profile/core; synced to Qdrant `knowledge_core`.
- Why: A dense, high-trust "our knowledge" export distinct from day-to-day honeypot.
- Related: [SERVICE:Qdrant], [CONCEPT:FourLayers]
- Injected facts:
  - [PATH:knowledge_core_db] data/knowledge_core.db is the M5 ripened concept store built by ripen-knowledge-core.py (residency + corroboration gated) and synced to the Qdrant knowledge_core collection.

### [PATH:memory_dir]
- What: Episodic logs.
- How: `memory/YYYY-MM-DD.md`.
- Use: Dream substrate; ingest receipts.
- Why: Cheap append-only daily record.
- Related: [CONCEPT:Episodic], [SYSTEM:Dream]
- Injected facts:
  - [PATH:memory_dir] memory/YYYY-MM-DD.md holds the append-only episodic day logs that feed the DreamEngine.

### [PATH:wiki_dir]
- What: Wiki synthesis pages.
- How: `wiki/` git-tracked; `index.md`, `log.md`.
- Use: Emit-only; grep search.
- Why: Browsable synthesis layer.
- Related: [CONCEPT:Wiki], [SYSTEM:WikiEngine]
- Injected facts:
  - [PATH:wiki_dir] wiki/ holds git-tracked emit-only synthesis pages (index.md, log.md) searched by grep, never re-ingested.

### [PATH:dreams_md]
- What: The dream consolidation report.
- How: `DREAMS.md`, appended by the DreamEngine.
- Use: "What happened overnight?" → read DREAMS.md.
- Why: Human-readable nightly status of consolidation.
- Related: [SYSTEM:Dream]
- Injected facts:
  - [PATH:dreams_md] DREAMS.md is the DreamEngine's nightly consolidation report; "what happened overnight" maps to reading it.

### [PATH:sessions_dir]
- What: Session distill input.
- How: `data/sessions/*.json`.
- Use: Read by SessionDistill at 02:15 UTC.
- Why: Drop chat exports here for distillation.
- Related: [SYSTEM:SessionDistill]
- Injected facts:
  - [PATH:sessions_dir] data/sessions/*.json is the SessionDistill input consumed nightly at 02:15 UTC.

### [PATH:synapse_bus]
- What: The Pi↔GZMO event bus.
- How: `data/Synapse/events.jsonl`, append-only, never consumed for state.
- Use: Tailed into episodic by synapse_pull.
- Why: Observability without state coupling.
- Related: [SYSTEM:SynapsePull], [CONCEPT:Pi]
- Injected facts:
  - [PATH:synapse_bus] data/Synapse/events.jsonl is the append-only Pi/GZMO bus, tailed into episodic by synapse_pull and never used as a state store.

### [PATH:backups]
- What: Purge snapshots.
- How: `data/backups/pre-full-purge-*` and `pre-nuclear-purge-*` (vault.db, neo4j-nodes.json, memory/, wiki/, DREAMS.md, eval reports, Synapse, Pi state).
- Use: Recovery after a purge.
- Why: Every destructive purge is reversible from here (Neo4j wipe would otherwise be permanent).
- Related: [HOWTO:NuclearPurge], [HOWTO:FullPurge]
- Injected facts:
  - [PATH:backups] data/backups/pre-(full|nuclear)-purge-<stamp>/ holds the pre-purge snapshot (vault.db, neo4j-nodes.json, memory, wiki, DREAMS.md, Synapse, Pi state) for recovery.

### [PATH:knowledge_dirs]
- What: The on-disk knowledge tree feeding ingest.
- How: `~/Schreibtisch/knowledge/` (watcher target), `archive/` (frozen migration source), `curated/` (the only ingest-eligible migration path).
- Use: Curated, consolidated docs go to `curated/`; raw migration material stays in `archive/` / sidecar staging.
- Why: Enforces curation-first: only condensed, operator-approved data is ingest-eligible.
- Related: [POLICY:CurationFirst], [POLICY:NoAutoMigration]
- Injected facts:
  - [PATH:knowledge_dirs] ~/Schreibtisch/knowledge/curated/ is the only ingest-eligible migration path; archive/ and sidecar-migration staging are frozen source material, never ingested raw.

---

## Domain 10 — Operator how-tos

### [HOWTO:VerifyProduction]
- What: Infra health check.
- How: `./scripts/verify-production.sh` — checks Prime, embed, Neo4j MCP, vault, FTS.
- Use: After reboot or infra change. Not an M4 eval gate.
- Why: Fast confirmation the plumbing is up before trusting recall.
- Related: [HOWTO:MemoryStatus], [SERVICE:Prime]
- Injected facts:
  - [HOWTO:VerifyProduction] ./scripts/verify-production.sh checks Prime/embed/Neo4j/vault/FTS health after a reboot or infra change; it is explicitly not an M4 eval gate.

### [HOWTO:MemoryStatus]
- What: Store count snapshot.
- How: `./scripts/memory-status.sh` — vault / honeypot / Qdrant counts.
- Use: Sanity-check populations.
- Why: Quick drift check between SQLite and Qdrant.
- Related: [HOWTO:SyncQdrant]
- Injected facts:
  - [HOWTO:MemoryStatus] ./scripts/memory-status.sh prints vault/honeypot/Qdrant counts for a quick population and drift check.

### [HOWTO:StartProduction]
- What: Bring the stack up.
- How: `./scripts/start-production.sh --daemon` — starts Prime, Pi embed, health, and the daemon.
- Use: Cold start.
- Why: One command to a running stack.
- Related: [SERVICE:Prime], [SYSTEM:Dream]
- Injected facts:
  - [HOWTO:StartProduction] ./scripts/start-production.sh --daemon starts Prime, Pi embed, health checks, and the gzmo daemon.

### [HOWTO:SyncQdrant]
- What: Manual Qdrant mirror.
- How: `./scripts/sync-vault-to-qdrant.sh` (or `.py --source honeypot`).
- Use: After ingest/seed to close the same-night sync gap.
- Why: Makes new honeypot facts searchable in Qdrant immediately instead of waiting for 01:45.
- Related: [SERVICE:Qdrant], [CONFIG:qdrant]
- Injected facts:
  - [HOWTO:SyncQdrant] After seeding or ingest, run ./scripts/sync-vault-to-qdrant.sh to mirror honeypot into Qdrant immediately rather than waiting for the 01:45 cron.

### [HOWTO:FullPurge]
- What: Standard memory reset.
- How: `./scripts/purge-all-memory.sh --confirm FULL_PURGE` — vault reset, Neo4j provenance strip, Qdrant honeypot+knowledge delete, episodic archive.
- Use: Before a slow wave re-ingest so the live store matches HEAD code.
- Why: Clears ingest footprint without nuking the entire graph.
- Related: [HOWTO:NuclearPurge], [HOWTO:WavePurge]
- Injected facts:
  - [HOWTO:FullPurge] ./scripts/purge-all-memory.sh --confirm FULL_PURGE resets vault, strips Neo4j ingest provenance, clears Qdrant honeypot+knowledge, and archives episodic; always dry-run and stop the daemon first.

### [HOWTO:NuclearPurge]
- What: True ground-zero clean slate.
- How: `./scripts/purge-all-memory.sh --confirm NUCLEAR_PURGE` — backs up, then wipes vault, the full Neo4j graph, all Qdrant collections (honeypot/knowledge/knowledge_core), knowledge_core.db, DREAMS.md, wiki/, the Synapse bus, the Redis distill queue, and Pi knowledge-state.json.
- Use: Stop daemon; `--dry-run --confirm NUCLEAR_PURGE` first; needs `scripts/.venv` (neo4j + redis drivers).
- Why: Exists for a complete rebuild under curation-first so no legacy/uncurated facts survive; backups first because the Neo4j wipe is otherwise irreversible.
- Related: [HOWTO:FullPurge], [POLICY:CurationFirst], [PATH:backups], [STATE:CleanSlate]
- Injected facts:
  - [HOWTO:NuclearPurge] A true clean slate is ./scripts/purge-all-memory.sh --confirm NUCLEAR_PURGE: it backs up then wipes vault, full Neo4j, all Qdrant collections, knowledge_core.db, DREAMS.md, wiki, Synapse, the Redis queue, and Pi state.
  - [HOWTO:NuclearPurge] Before a nuclear purge, stop the daemon and run --dry-run; it needs scripts/.venv with the neo4j and redis drivers or the Neo4j wipe and Redis flush are skipped.

### [HOWTO:WavePurge]
- What: Wave-scoped ingest removal.
- How: `./scripts/purge-wave-ingest.sh <wave> --confirm PURGE` — removes one wave's footprint from vault, Neo4j, Qdrant, and episodic by `source_file LIKE 'wave_NN_%'`.
- Use: Undo a single migration wave.
- Why: Surgical rollback without a full reset.
- Related: [HOWTO:FullPurge]
- Injected facts:
  - [HOWTO:WavePurge] ./scripts/purge-wave-ingest.sh <wave> --confirm PURGE removes a single wave's footprint (vault/Neo4j/Qdrant/episodic) matched by source_file prefix.

### [HOWTO:SeedCore]
- What: Populate core self-knowledge.
- How: `scripts/seed-core-stack.py` injects the `[TYPE:Name]` facts from this document into vault + honeypot (Structural, conf 0.95); then `gzmo memory embed` and `sync-vault-to-qdrant.py --source honeypot`.
- Use: After a clean slate, before any migration data.
- Why: Gives the empty system a complete, low-risk understanding of its own machine first.
- Related: [STATE:CleanSlate], [POLICY:CurationFirst]
- Injected facts:
  - [HOWTO:SeedCore] scripts/seed-core-stack.py injects this document's core facts into vault+honeypot (Structural, conf 0.95, source manual/core_stack_20260607.md); follow with gzmo memory embed and sync-vault-to-qdrant.py --source honeypot.

---

## Domain 11 — Operating doctrine

### [POLICY:CurationFirst]
- What: Only curated, consolidated data enters memory.
- How: Raw Takeout / `00_inbox` / `01_ingest_ready` are read-only source; only near-perfect, consolidated docs promoted to `knowledge/curated/` are ingest-eligible.
- Use: Curate and condense before any injection.
- Why: Protects the verified stores from low-quality bulk data; quality is the bottleneck, not volume.
- Related: [POLICY:NoAutoMigration], [PATH:knowledge_dirs], [CONFIG:ingest]
- Injected facts:
  - [POLICY:CurationFirst] GZMO never ingests raw migration-pile data; migration material is first curated into a near-perfect consolidated state, and only that condensed data is injected.

### [POLICY:NoAutoMigration]
- What: No automated re-migration pipeline.
- How: Bulk/reactive ingest paths (watcher, trigger-wave, batch-ingest) are not used for migration; injection is a deliberate, manual operator act.
- Use: Do not build or run automated wave re-ingest.
- Why: Keeps a human in the loop for everything that becomes long-term memory.
- Related: [POLICY:CurationFirst], [PATH:knowledge_dirs]
- Injected facts:
  - [POLICY:NoAutoMigration] There is no automated re-migration; curated data is injected by deliberate operator action, never by bulk watcher/trigger/batch ingest.

### [POLICY:NoBulkSwap]
- What: No engine swap or bulk ingest without an eval gate.
- How: Model swaps (e.g. Gemma 4) require the quality gate; corpus expansion requires replay/recall gates.
- Use: Gate before changing Prime or feeding a wave.
- Why: Prevents silent quality regressions in cognition or recall.
- Related: [MODEL:Gemma4Cutover], [CONFIG:ingest]
- Injected facts:
  - [POLICY:NoBulkSwap] Never swap the Prime engine or bulk-ingest a corpus without passing the relevant eval gate (turbo-quality-gate.sh for models; replay/recall for corpora).

---

## Domain 12 — Current state

### [STATE:CleanSlate]
- What: The system was reset to ground zero on 2026-06-07.
- How: A NUCLEAR_PURGE wiped vault (was ~1039/809/801), Neo4j (987 nodes/1208 rels), and all Qdrant collections (honeypot 698, knowledge 500, knowledge_core 78) to zero; derived stores archived to `data/backups/pre-nuclear-purge-20260607-183103/`.
- Use: Populate core self-knowledge first (seed-core-stack.py), then curated migration data only.
- Why: Establish a clean, fully-curated memory from scratch.
- Related: [HOWTO:NuclearPurge], [HOWTO:SeedCore], [STATE:EnginesDisabled]
- Injected facts:
  - [STATE:CleanSlate] On 2026-06-07 GZMO was reset to a true clean slate via NUCLEAR_PURGE (vault, Neo4j, all Qdrant collections zeroed; backup at data/backups/pre-nuclear-purge-20260607-183103); core self-knowledge is populated first.

### [STATE:EnginesDisabled]
- What: Cognition engines and the ingest watcher are off during rebuild.
- How: `[dreams]`, `[spark]`, `[session_distill]` set `enabled = false`; `inbox_ingest` watcher `disabled = true`.
- Use: Re-enable after curated population + a passing baseline.
- Why: Prevents background writes from polluting the freshly-seeded memory.
- Related: [STATE:CleanSlate], [SYSTEM:Dream], [SYSTEM:Spark], [SYSTEM:SessionDistill]
- Injected facts:
  - [STATE:EnginesDisabled] During the clean-slate rebuild, [dreams]/[spark]/[session_distill] are enabled=false and the inbox_ingest watcher is disabled to keep background engines from polluting freshly-seeded memory.

---

*End of GZMO Core Stack Knowledge. Extend by adding cards in this exact format; re-seed with `scripts/seed-core-stack.py`.*
