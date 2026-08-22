# MemoryArena Embed-Path Baseline — REAL Router + Qdrant
**Date:** 2026-08-22
**Method:** Embed query via router `http://192.168.31.110:8081/v1/embeddings` (gzmo-embed, Qwen3-Embedding-0.6B-Q8_0, 1024-dim) → vector search in Qdrant `127.0.0.1:6333` collection `honeypot` (685 points)
**Qdrant reachable:** True
**Embed router reachable:** True
**Score (top-1):** 8/12 hits
**Score (top-5):** 9/12 hits
**Comparison:** keyword-only baseline was 3/12

---

| ID | Category | Question | Method | Hit? (top-1) | Hit? (top-5) | Score | Top Hit | Keywords Found |
|---|---|---|---|---|---|---|---|---|
| Q1 | single-fact | What is CT101's role in the GZMO architecture? | embed | NO | NO | 0.7057 | [SYSTEM:Session] GZMO is a daemon-based AI ecosystem with a distillation pipelin | none |
| Q2 | single-fact | What is the dual-writer rule? Can two overnight writers run  | embed | NO | NO | 0.4583 | [PATH:vault_db] Living SQLite authority is data-next/vault.db (semantic_vault +  | none |
| Q3 | single-fact | What is the Prime inference server and what port does it run | embed | YES | YES | 0.7581 | [SERVICE:Prime] Production cognition for the living instance is the OpenAI-compa | Prime, 8000, 127.0.0.1:8000, OpenAI-compatible |
| Q4 | single-fact | What does Obolus do in the AOS energy routing chain? | embed | YES | YES | 0.7305 | [CONCEPT:Obolus] Obolus/task routing and metering lessons from CT101 inform Gate | Obolus |
| Q5 | single-fact | What are the stages of the GZMO distillation pipeline? | embed | YES | YES | 0.9115 | [DECISION:Session] GZMO uses a distillation pipeline with stages: prep → extract | extract, verify, promote, honeypot |
| Q6 | multi-session | ADR-0003 originally said CT101 is frozen reference, then ADR | embed | YES | YES | 0.7461 | [POLICY:ADR0003] ADR-0003 (2026-07-16): the workstation is the sole living insta | CT101, workstation |
| Q7 | multi-session | ADR-0003 said one writer, ADR-0004 said airgap, ADR-0007 sai | embed | NO | NO | 0.5315 | [DECISION:ADR0003] Product gate is gzmo status answering did-last-night-work via | none |
| Q8 | multi-session | How does a TinyFolder drop reach the living vault? Trace the | embed | NO | YES | 0.5950 | [PROJECT:Quillhorn Cascade] Has a calibration ledger stored at vault path ledger | none |
| Q9 | multi-session | If a beat-gate passes for one loop, what must happen before  | embed | YES | YES | 0.6703 | [LESSON:BeatGates] Beat-gates (config/ops/cognition/knowledge smokes) are the ho | beat-gate |
| Q10 | multi-session | On 2026-07-15, a cutover happened. Was the vault imported fr | embed | YES | YES | 0.8384 | [POLICY:CT101Frozen] Cutover 2026-07-15 put production on the workstation with f | cutover, 2026-07-15, fresh, 60k-fact, no vault import |
| Q11 | single-fact | What are the roles of Qdrant, Neo4j, and SQLite in GZMO? | embed | YES | YES | 0.8277 | [TOOL:Session] GZMO ecosystem includes Redis (:6379), Qdrant (:6333), Neo4j (:76 | Qdrant, Neo4j |
| Q12 | single-fact | Is the Chaos Engine required for metabolism to function? | embed | YES | YES | 0.7082 | [SYSTEM:Chaos] Chaos PulseLoop is opt-in for stdio chat and forced live in gzmo  | chaos, opt-in, metabolism, not depend |

---

## Detailed results

### Q1 — single-fact

**Question:** What is CT101's role in the GZMO architecture?

**Method:** embed

**Hit (top-1):** NO

**Hit (top-5):** NO

**Top hit (score=0.7057):** [SYSTEM:Session] GZMO is a daemon-based AI ecosystem with a distillation pipeline (prep → extract → verify → promote → honeypot → ripen) for memory consolidation.

**Keywords found:** none

**Notes:** Found keywords: none; score=0.7057; top5_any_hit=False

### Q2 — single-fact

**Question:** What is the dual-writer rule? Can two overnight writers run on the same vault?

**Method:** embed

**Hit (top-1):** NO

**Hit (top-5):** NO

**Top hit (score=0.4583):** [PATH:vault_db] Living SQLite authority is data-next/vault.db (semantic_vault + honeypot + evidence); stop writers before destructive maintenance.

**Keywords found:** none

**Notes:** Found keywords: none; score=0.4583; top5_any_hit=False

### Q3 — single-fact

**Question:** What is the Prime inference server and what port does it run on?

**Method:** embed

**Hit (top-1):** YES

**Hit (top-5):** YES

**Top hit (score=0.7581):** [SERVICE:Prime] Production cognition for the living instance is the OpenAI-compatible server at http://127.0.0.1:8000/v1 (chat, ingest extract/verify, dream, spark).

**Keywords found:** Prime, 8000, 127.0.0.1:8000, OpenAI-compatible

**Notes:** Found keywords: Prime, 8000, 127.0.0.1:8000, OpenAI-compatible; score=0.7581; top5_any_hit=True

**Top-5 hit 1 (score=0.7581):** [SERVICE:Prime] Production cognition for the living instance is the OpenAI-compatible server at http://127.0.0.1:8000/v1 (chat, ingest extract/verify, dream, spark).

**Top-5 hit 2 (score=0.6976):** [SERVICE:Prime] systemd unit llama-prime.service may be inactive while a manual llama-server still answers on :8000 — judge LLM health by /v1/models, not unit LED alone.

### Q4 — single-fact

**Question:** What does Obolus do in the AOS energy routing chain?

**Method:** embed

**Hit (top-1):** YES

**Hit (top-5):** YES

**Top hit (score=0.7305):** [CONCEPT:Obolus] Obolus/task routing and metering lessons from CT101 inform GatewayRouter task kinds; living chat still centers the active engine profile for primary turns.

**Keywords found:** Obolus

**Notes:** Found keywords: Obolus; score=0.7305; top5_any_hit=True

**Top-5 hit 1 (score=0.7305):** [CONCEPT:Obolus] Obolus/task routing and metering lessons from CT101 inform GatewayRouter task kinds; living chat still centers the active engine profile for primary turns.

**Top-5 hit 2 (score=0.7024):** [SYSTEM:Obolus] Task routing and metering lessons from CT101 inform GatewayRouter task kinds

### Q5 — single-fact

**Question:** What are the stages of the GZMO distillation pipeline?

**Method:** embed

**Hit (top-1):** YES

**Hit (top-5):** YES

**Top hit (score=0.9115):** [DECISION:Session] GZMO uses a distillation pipeline with stages: prep → extract → verify → promote → honeypot → ripen.

**Keywords found:** extract, verify, promote, honeypot

**Notes:** Found keywords: extract, verify, promote, honeypot; score=0.9115; top5_any_hit=True

**Top-5 hit 1 (score=0.9115):** [DECISION:Session] GZMO uses a distillation pipeline with stages: prep → extract → verify → promote → honeypot → ripen.

**Top-5 hit 2 (score=0.8330):** [SYSTEM:Session] GZMO has a full distillation pipeline (prep → extract → verify → promote → honeypot → ripen) in gzmo-core but has never been fed data.

### Q6 — multi-session

**Question:** ADR-0003 originally said CT101 is frozen reference, then ADR-0005 amended this. What is the current state of CT101 vs workstation living-host placement?

**Method:** embed

**Hit (top-1):** YES

**Hit (top-5):** YES

**Top hit (score=0.7461):** [POLICY:ADR0003] ADR-0003 (2026-07-16): the workstation is the sole living instance; CT101 is a frozen reference machine, not a permanent dual-stack product.

**Keywords found:** CT101, workstation

**Notes:** Found keywords: CT101, workstation; score=0.7461; top5_any_hit=True

**Top-5 hit 1 (score=0.7461):** [POLICY:ADR0003] ADR-0003 (2026-07-16): the workstation is the sole living instance; CT101 is a frozen reference machine, not a permanent dual-stack product.

**Top-5 hit 2 (score=0.5859):** [POLICY:CT101Frozen] CT101 is frozen legacy: leave it alone unless explicitly debugging CT101 itself; never edit CT101 gzmo.toml to point loops at lab scripts.

### Q7 — multi-session

**Question:** ADR-0003 said one writer, ADR-0004 said airgap, ADR-0007 said no lite SKU. What is the current product story?

**Method:** embed

**Hit (top-1):** NO

**Hit (top-5):** NO

**Top hit (score=0.5315):** [DECISION:ADR0003] Product gate is gzmo status answering did-last-night-work via data-next/scheduler-runs/ plus vault/honeypot counts

**Keywords found:** none

**Notes:** Found keywords: none; score=0.5315; top5_any_hit=False

### Q8 — multi-session

**Question:** How does a TinyFolder drop reach the living vault? Trace the path through Brain Feed.

**Method:** embed

**Hit (top-1):** NO

**Hit (top-5):** YES

**Top hit (score=0.5950):** [PROJECT:Quillhorn Cascade] Has a calibration ledger stored at vault path ledger://quillhorn/cascade-v1

**Keywords found:** none

**Notes:** Found keywords: none; score=0.5950; top5_any_hit=True

**Top-5 hit 1 (score=0.5643):** [CONCEPT:FourLayers] Memory has four layers: vault (all verified facts), honeypot (Tier-1 curated), evidence (Tier-2 spans), knowledge_core (M5 ripened); Qdrant mirrors honeypot only.

**Top-5 hit 2 (score=0.5636):** [HOWTO:SeedCoreInsight] Prefill living memory with scripts/seed-core-stack.py --doc docs/CORE_INSIGHT.md --db data-next/vault.db --source-file manual/core_insight_20260717.md then gzmo memory embed an

### Q9 — multi-session

**Question:** If a beat-gate passes for one loop, what must happen before it lands in the living host?

**Method:** embed

**Hit (top-1):** YES

**Hit (top-5):** YES

**Top hit (score=0.6703):** [LESSON:BeatGates] Beat-gates (config/ops/cognition/knowledge smokes) are the honesty layer before trusting a composed runtime — fixtures first, --live second.

**Keywords found:** beat-gate

**Notes:** Found keywords: beat-gate; score=0.6703; top5_any_hit=True

**Top-5 hit 1 (score=0.6703):** [LESSON:BeatGates] Beat-gates (config/ops/cognition/knowledge smokes) are the honesty layer before trusting a composed runtime — fixtures first, --live second.

**Top-5 hit 2 (score=0.5410):** [LESSON:DiscoveryCaution] Do not add DiscoveryEngine into living gzmo-scheduler/serve overnight until fixture beat-gates stay green.

### Q10 — multi-session

**Question:** On 2026-07-15, a cutover happened. Was the vault imported from CT101 or fresh data-next?

**Method:** embed

**Hit (top-1):** YES

**Hit (top-5):** YES

**Top hit (score=0.8384):** [POLICY:CT101Frozen] Cutover 2026-07-15 put production on the workstation with fresh data-next/ — no vault import from CT101's 60k-fact store.

**Keywords found:** cutover, 2026-07-15, fresh, 60k-fact, no vault import

**Notes:** Found keywords: cutover, 2026-07-15, fresh, 60k-fact, no vault import; score=0.8384; top5_any_hit=True

**Top-5 hit 1 (score=0.8384):** [POLICY:CT101Frozen] Cutover 2026-07-15 put production on the workstation with fresh data-next/ — no vault import from CT101's 60k-fact store.

**Top-5 hit 2 (score=0.6538):** [STATE:LivingCutover] As of 2026-07-15/16 cutover, production is the workstation living instance with data-next/, gzmo-serve overnight, and CT101 frozen.

### Q11 — single-fact

**Question:** What are the roles of Qdrant, Neo4j, and SQLite in GZMO?

**Method:** embed

**Hit (top-1):** YES

**Hit (top-5):** YES

**Top hit (score=0.8277):** [TOOL:Session] GZMO ecosystem includes Redis (:6379), Qdrant (:6333), Neo4j (:7687) as sidecar services.

**Keywords found:** Qdrant, Neo4j

**Notes:** Found keywords: Qdrant, Neo4j; score=0.8277; top5_any_hit=True

**Top-5 hit 1 (score=0.8277):** [TOOL:Session] GZMO ecosystem includes Redis (:6379), Qdrant (:6333), Neo4j (:7687) as sidecar services.

**Top-5 hit 2 (score=0.8140):** [SYSTEM:Session] GZMO ecosystem uses local inference (Ornith 35B Q4_K_M on localhost:8000) with sidecar services: Redis, Qdrant, Neo4j.

### Q12 — single-fact

**Question:** Is the Chaos Engine required for metabolism to function?

**Method:** embed

**Hit (top-1):** YES

**Hit (top-5):** YES

**Top hit (score=0.7082):** [SYSTEM:Chaos] Chaos PulseLoop is opt-in for stdio chat and forced live in gzmo --repl ops console; metabolism does not depend on chaos.

**Keywords found:** chaos, opt-in, metabolism, not depend

**Notes:** Found keywords: chaos, opt-in, metabolism, not depend; score=0.7082; top5_any_hit=True

**Top-5 hit 1 (score=0.7082):** [SYSTEM:Chaos] Chaos PulseLoop is opt-in for stdio chat and forced live in gzmo --repl ops console; metabolism does not depend on chaos.

**Top-5 hit 2 (score=0.6674):** [SYSTEM:ledger://quillhorn/cascade-v1] Operates using nightburst metabolism rather than calendar soak


---

## Comparison: keyword-only vs embed-path

**Keyword-only baseline (BASELINE.md):** 3/12 hits (Q4 Obolus, Q8 TinyFolder/distill, Q12 chaos).

**Embed-path baseline (this file):** 8/12 top-1 hits, 9/12 top-5 hits.

**Questions gained (keyword→embed):** ['Q10', 'Q11', 'Q3', 'Q5', 'Q6', 'Q9']

**Questions lost (keyword→embed):** ['Q8']

**Questions in top-5 but not top-1:** ['Q8']

### Interpretation

The embed-path uses the REAL production embedding model (Qwen3-Embedding-0.6B) to embed each question and search Qdrant by cosine similarity, rather than the keyword-matching approach of the original baseline. This tests whether the semantic embed can surface the correct honeypot entries that keyword search missed.

If the embed-path scores similarly or worse than keyword-only, the system's weakness is in **retrieval ranking** — the correct content may not be in the top hit even with semantic search, suggesting the honeypot collection lacks the right document chunks or the embedding model doesn't discriminate well for these architectural questions.

If the embed-path scores better, the weakness was in the **keyword matching** of the original baseline — semantic similarity surfaces relevant chunks that exact keyword matching missed. This would imply the MemoryLake HOLD should focus on improving fact-coverage (what gets ingested into the honeypot) rather than retrieval ranking.
