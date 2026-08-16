# Lineage SOTA — 2026-08-16 arXiv tourism

**Host:** telescope. This is harvest, not a second ship.  
**Active bet unchanged:** `felt-use-mass-growth`.  
**Does not:** start `gzmo serve`, clone a Mem* SKU, or run immune apply on living.

Same-day delta after the three-plane atlas. Query: arXiv API + HTML abstracts, newest first. Most 2026 memory papers are **SKU / RL / survey**. One rule was missing on the retrieve path.

---

## Steal vs park (this sitting)

| Paper | Date | Steal | Non-steal |
|-------|------|-------|-----------|
| **Governed Persistent Memory** ([2608.12476](https://arxiv.org/abs/2608.12476)) | 2026-08-12 | Retrieval ≠ public eligibility. Project **assertable** (`is_latest`) **before** rank. Fail-closed release. | Multi-tenant OS, 7B “governed lane” as USP |
| **Temporal Validity in Retrieval Memory** ([2606.26511](https://arxiv.org/abs/2606.26511)) | 2026-06-25 | Stale + current vectors score similarly; drop superseded from the index | New graph RAG product |
| **Manufactured Confidence** ([2606.29279](https://arxiv.org/abs/2606.29279)) | 2026-06-28 | Compression must not mint confident facts (already the honeypot gate) | Extra judge model |
| **Memory Provenance Laundering** ([2607.29167](https://arxiv.org/abs/2607.29167)) | 2026-07-31 | Origin stays bound through consolidate | Crypto firewall crate |
| **When Memory Becomes Authority** ([2608.01679](https://arxiv.org/abs/2608.01679)) | 2026-08-03 | Consolidation is an authorization boundary | New bench as soak GREEN |
| **Explicit, Not Longer** ([2608.06953](https://arxiv.org/abs/2608.06953)) | 2026-08-07 | Epistemic stance / confidence must survive compact | Prompt-length theater |
| **Memory Reward Inflation** ([2608.00017](https://arxiv.org/abs/2608.00017)) | 2026-06-29 | Do not mint Q from self-scores (Glance Q=0 already) | Gym reward loops |
| **VerMem** ([2608.03137](https://arxiv.org/html/2608.03137)) | 2026-08 | Local+global verify cousin | RL curriculum / extra verifiers at runtime |
| **AgeMem** ([2601.01885](https://arxiv.org/abs/2601.01885), ACL 2026) | 2026-01 | Memory ops as tools (we have `memory_*`) | GRPO / agent-chosen page-in as the product |
| **LightMem** ([2510.18866](https://arxiv.org/abs/2510.18866)) | 2025-10 | Sleep-time update decoupled from online (we have daemon) | Atkinson–Shiffrin SKU |
| **TRUSTMEM** ([2606.25161](https://arxiv.org/abs/2606.25161)) | 2026-06 | Trustworthy write/revise/delete | Learned write policy overnight |
| **Retain or Consolidate?** ([2607.17545](https://arxiv.org/abs/2607.17545)) | 2026-07-20 | Budget chooses retain vs compact (G6 cousin) | New operator crate |
| **ERSkill** ([2608.12720](https://arxiv.org/abs/2608.12720)) | 2026-08-13 | Retrieval policy can evolve with skills | Auto-evolve retrieve without pin |
| Survey [2603.07670](https://arxiv.org/abs/2603.07670) | 2026-03 | Taxonomy only | Re-found the Keep |
| G-Memory / graph-memory surveys | 2025–26 | — | Multi-agent graph SKU |

Already in the atlas (do not re-queue): MemRL, Memento, A-Mem, Auto-Dreamer, Memory as Metabolism, SleepGate, SuperLocalMemory, ACE, Agent Memory characterization (2606.06448).

---

## GZMO hole this sitting closed

SQLite hybrid recall already hydrates with `is_latest = 1`. Qdrant did **not**:

1. Rank lists could fill prefetch with superseded point ids (hydrate dropped them later; they still stole slots).
2. Nightly upsert of latest rows did not delete superseded vectors.

**Graft (not a new crate):** `filter_assertable_honeypot_ids` before RRF; `sync-vault-to-qdrant.py` prunes orphans after honeypot upsert.

**T1 follow-up (this sitting):** filtering Qdrant ids *after* a `PREFETCH_K` search starved the vector list whenever stale points occupied slots. Recall now overfetches `QDRANT_PREFETCH_K` (`PREFETCH_K * 2`), drops superseded ids, then truncates. Honeypot upsert stamps `is_latest: true` on payload. Search still has **no** Qdrant payload filter — enabling it before a living re-sync would drop every old point that lacks the field.

---

## Watch items (still not active)

0. Living felt-use mass + soak nights 2–3 (`felt-use-mass-growth`).  
1. Immune apply on living (lab until soak).  
2. ACE helpful/harmful counters on `SKILL.md` bullets (optional L1 follow-up; pin still required).

Park: AgeMem RL, VerMem training verifiers, LightMem/Mem0/Letta SKU, overnight LoRA, second writer, Observatory glass.
