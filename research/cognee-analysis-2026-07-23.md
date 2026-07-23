# Cognee analysis → GZMO living/memory (2026-07-23)

Research capture of [topoteretes/cognee](https://github.com/topoteretes/cognee) (v1.4.0 tree, Apache-2.0) against GZMO airgap / Brain Feed / nutrient USP. **Borrow algorithms and UX patterns; do not adopt Cognee as a second overnight writer or cloud-default memory SKU.**

**Doctrine:** [ADR-0003](../docs/ADR-0003-one-instance-metabolism.md) · [ADR-0004](../docs/ADR-0004-airgap-living-usp.md) · [ADR-0005](../docs/ADR-0005-flywheel-over-frozen-topology.md)  
**Sibling field notes:** [sleep-consolidation-sota-2026-07-22.md](./sleep-consolidation-sota-2026-07-22.md) · [living-external-attach-plug-and-play-2026-07-22.md](./living-external-attach-plug-and-play-2026-07-22.md)

---

## Verdict

**Cognee is a polished, agent-facing knowledge-graph memory SDK** (Python-first, ~29k★): ingest → cognify (LLM entity/relation extract) → hybrid vector+graph recall → optional session→permanent “improve/distill.” It overlaps GZMO’s *language* (remember / recall / forget / improve) but solves a different product: **company/agent RAG memory with multi-tenant SaaS gravity**, not **sovereign overnight vault metabolism on one living box**.

For GZMO: **skip as stack; steal a few consolidation UX ideas.** Replacing honeypot/Qdrant/Brain Feed with Cognee would fight ADR-0003/0004 and add LLM-heavy cognify cost to every write.

---

## What it is (primary sources)

| Fact | Source |
|------|--------|
| Open-source “AI memory platform for agents”; self-hosted KG engine | README |
| License Apache-2.0 | `LICENSE`, `pyproject.toml` |
| Version sampled | `pyproject.toml` → `1.4.0` |
| Paper | [arXiv:2505.24478](https://arxiv.org/abs/2505.24478) (KG↔LLM interface) |
| Docs hub | https://docs.cognee.ai/ · architecture: relational + vector + graph |

### Core loop

V2 API mental model (README + `cognee/api/v1/{remember,recall,forget,improve}`):

1. **`remember(data)`** — permanent: `add` + `cognify` (+ optional `improve` / self-improvement). With `session_id`: fast session cache; default `self_improvement=True` bridges into permanent graph in background.
2. **`recall(query)`** — auto-routed search over session then graph/vector; many `SearchType`s (`RAG_COMPLETION` default, hybrid, graph COT, temporal, lexical/BM25, etc.).
3. **`improve(dataset, session_ids=…)`** — feedback weights on graph edges/nodes → persist session Q&A → **session distillation** (curator/writer LLM lessons → `session_learnings`) → triplet enrichment → optional global context index / truth subspace.
4. **`forget(...)`** — delete data item / dataset / everything; `memory_only` keeps raw files for re-cognify.

Legacy/power path still exported: `add` / `cognify` / `memify` / `search` / `prune`.

### Data model (rough)

- **Relational:** documents, chunks, provenance, users/tenants/datasets (SQLAlchemy; default SQLite).
- **Vector:** embeddings of chunks / DataPoints (default LanceDB; prod pitch: pgvector).
- **Graph:** entities/relations (default **Ladybug**, Kuzu-lineage; Neo4j / Neptune / Postgres-graph optional).
- **Session cache:** QA / agent traces / feedback (default SQLite; Redis / Postgres / fs / tapes).
- Typed memory entries: `MemoryEntry`, `QAEntry`, `TraceEntry`, `FeedbackEntry`, skill-run entries; migration sources Mem0 / Letta / Zep-Graphiti / COGX archive.

### Storage backends

| Layer | Default (local) | First-party / recommended | Community / external |
|-------|-----------------|---------------------------|----------------------|
| Relational | SQLite | Postgres | Turso |
| Vector | LanceDB | pgvector | Qdrant, Chroma, Weaviate, Milvus via [cognee-community](https://github.com/topoteretes/cognee-community) |
| Graph | Ladybug (Kuzu-compat) | Neo4j, Neptune, Postgres graph | — |
| Session | SQLite | Redis, Postgres | fs, tapes |

README also sells **“whole memory layer on Postgres”** (graph + pgvector + session + metadata) for ops simplicity — still a different topology than GZMO’s vault.db + honeypot + Qdrant + optional Neo4j.

### LLM / embed deps

- Core deps: `openai`, `litellm`, `instructor`, `tiktoken` (`pyproject.toml`).
- Defaults (`LLMConfig` / `EmbeddingConfig`): **`openai/gpt-5-mini`**, embed **`openai/text-embedding-3-large`** (3072-d fallback).
- Optional: Ollama, Anthropic, Azure, Groq, llama-cpp, HuggingFace, fastembed, Mistral, BAML.
- Cognify / improve / session distill are **LLM-write-amplified** (extract + curator + writer stages). Airgap needs local provider + embed rewiring; not the default path.
- Telemetry: `send_telemetry` unless `TELEMETRY_DISABLED` or `ENV` in `{test,dev}` — sends anonymous/persistent machine IDs (+ hashed API-key tracking id). Airgap boxes should disable.

### MCP / agents

- Docker MCP image `cognee/cognee-mcp`; CLI UI path also Docker-dependent.
- Claude Code / OpenClaw plugins; Rust + TS clients; Cloud `serve()` redirect.
- Product gravity: multi-user, tenants, FastAPI, Cognee Cloud — opposite of ADR-0004 “reject public multi-tenant webserver SKU.”

---

## Comparison table (honest)

| System | Core idea | Overnight / sleep | Utility / forget | Fit vs GZMO living |
|--------|-----------|-------------------|------------------|--------------------|
| **GZMO** | Vault → distill → promote → honeypot; nutrient/Brain Feed; MCP attach; single writer | Daemon metabolism on claimed host | `utility_score` + immune forget/apply (G4–G5 craft) | **USP owner** |
| **Cognee** | Doc/session → LLM cognify → vector+graph; remember/recall/improve | Session→graph improve/distill (LLM-heavy, often online) | `feedback_weight` / `importance_weight` / `feedback_influence`; forget = delete/re-cognify | Overlap of verbs, not of metabolism |
| **MemGPT / Letta** | OS paging: core vs archival context | Eviction / tier policy | Context budget, not vault utility | Steal tier paging; already on sleep-SOTA cite list |
| **MemoryOS** | STM → MTM → LPM heat promote | Heat-based promote | Heat ≠ MemRL utility | Steal promote heat; not a graph cognify stack |
| **MemRL** | Intent–Experience–Utility; two-phase retrieval | Policy evolution without weight updates | Explicit utility | Closest to GZMO G4; Cognee feedback_weight is a weaker cousin |

Cognee is closest to **Mem0 + Graphiti-shaped KG memory** with better packaging than to MemRL or GZMO’s disposable-vault doctrine.

---

## Strengths for a GZMO-like use

- Clean **four-verb API** and session vs permanent split (easy agent mental model).
- Real **hybrid retrieval** menu (vector, graph, BM25, temporal, agentic).
- **Session distillation** pipeline (curate → accept/reject → cognify lessons) is a concrete offline consolidation recipe.
- **Feedback → edge weight** loop (`improve` + `feedback_influence`) — value-aware retrieval without LoRA.
- Local defaults exist (SQLite + LanceDB + Ladybug); Ollama/fastembed extras; Apache-2.
- Migration importers (Mem0/Letta/…) — useful if ever ingesting external agent dumps into *lab*, not living.

## Weaknesses for GZMO living (airgap, single-writer)

- **Default cloud LLM/embed** — cognify cost and network assumption clash with ADR-0004 airgap honesty.
- **Qdrant is community-adapter**, not first-class — GZMO already standardized on Qdrant for living vectors.
- **Second writer risk** if run beside CT101 metabolism (ADR-0003). Same vault story must not get a parallel cognify daemon.
- **Python mega-framework** (FastAPI users, tenants, Modal/Railway/Cloud) — ops and attack surface vs Rust `gzmo-core` + thin MCP.
- **Telemetry on by default** outside test/dev — bad for honeypot / airgap.
- **Forget ≠ immune value-forgetting** — Cognee forget is CRUD delete / memory_only re-cognify, not SCM-style value decay.
- **No Brain Feed / nutrient / promote-by-loop / beat-gate** — different organism.

---

## Overlap vs differentiate

| Concern | Cognee | GZMO |
|---------|--------|------|
| Persist agent context | remember/recall datasets | vault facts → honeypot; MCP search/recall scratch |
| Consolidate sessions | improve + session_distillation | distill → promote → embed; dream/spark |
| Value-aware retrieve | feedback_weight / importance_weight | `utility_score` + reinforce_by (MemRL-shaped) |
| Forget | delete dataset / memory_only | `gzmo immune forget` + capped apply |
| Attach | MCP Docker / Cloud / plugins | `gzmo-living` / `gzmo-memory` labeled stdio attach |
| Graph | first-class KG product | Neo4j/wiki optional; honeypot is cortex |
| Writer | any process with API key | **one** overnight writer under mutex |

**Differentiate:** GZMO wins on airgap USP, single-writer doctrine, nutrient/Brain Feed, immune, promote flywheel. Cognee wins on out-of-box multimodal cognify + graph search productization for *multi-agent company brain* demos.

---

## Steal-or-skip

### Steal (concrete)

1. **Session→lesson distillation shape** — curator batch → writer/rejecter → tagged distillate node set (`session_learnings`). Map onto vault→vault / promote docs, not onto a second graph writer. Source: `cognee/modules/session_distillation/distill.py`.
2. **Feedback influence on retrieval** — scalar bump/penalize of supporting nodes after session ratings. Cross-check with MemRL utility (prefer MemRL semantics; steal Cognee’s *apply feedback then re-rank* ops story). Source: `improve()` stages + `feedback_influence` on retrievers.
3. **Four-verb agent surface** — keep GZMO MCP verbs aligned (`search`/`recall`/`turn_start` already); optionally document remember/forget aliases in attach docs for agent UX parity — **without** exposing Cognify writes to attach clients.
4. **`memory_only` reprocess** — ability to drop derived index and rebuild from raw — useful metaphor for “re-embed / re-promote without trash vault.”
5. **Dry-run token estimate before cognify-class work** — `remember(..., dry_run=True)` pattern for nutrient budgeting before heavy distill.

### Skip / leave alone

1. **Adopt Cognee as living memory engine** — dual-writer + Python stack + cloud defaults.
2. **Ladybug/LanceDB swap for Qdrant/honeypot** — no USP gain; migration tax.
3. **Cognee Cloud / public MCP HTTP** — ADR-0004 out of brand.
4. **Tenant/FastAPI-users multi-tenancy** — not the living box.
5. **BEAM benchmark chasing via cognify** — borrow-eval only if we need agent-memory leaderboards; not a metabolism gate.
6. **Community Qdrant adapter as “we already use Qdrant” excuse to vendor Cognee** — false equivalence.

### Borrow-next ranking (relative to sleep-SOTA list)

| Priority | Idea | vs existing leaps |
|----------|------|-------------------|
| Higher | Session distill curator/writer pattern for promote docs | Complements G3 promote+embed |
| Medium | Feedback→weight → reinforce_by story polish | Subsumed by G4 utility if reinforced from real use |
| Low | SearchType menu / FEELING_LUCKY-style routers | Only if recall quality plateaus |
| None | Whole Cognee deploy | Explicit non-goal |

---

## Explicit non-goals

- Overnight Cognee cognify writer next to CT101 daemon.
- Replacing honeypot with Cognee graph as cortex.
- Cloud LLM required for core remember path.
- Public multi-tenant Cognee SKU narrative in GZMO docs.

---

## Attribution / method

- Shallow clone `/tmp/cognee-analysis` from `https://github.com/topoteretes/cognee` (main tip 2026-07-23).
- Read README, LICENSE, `pyproject.toml`, `cognee/__init__.py`, `api/v1/{remember,recall,forget,improve}`, `modules/session_distillation/distill.py`, DB configs under `infrastructure/databases/*/config.py`, docs architecture page.
- GZMO context: ADR-0003/0004, sleep-consolidation SOTA note, living attach note; MCP memory search (no prior Cognee facts).

When porting snippets: keep Apache-2 notices; prefer reimplementation in `gzmo-core` over vendoring.
