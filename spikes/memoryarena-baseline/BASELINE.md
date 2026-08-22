# MemoryArena Baseline — Current GZMO Memory System

**Date:** 2026-08-22
**Qdrant reachable:** True
**Collection:** honeypot (685 points, 1024-dim Cosine)
**Method:** SQLite FTS over `data-next/vault.db` honeypot table (backs the Qdrant collection)
**Fallback:** keyword grep over `data-next/` + `docs/`

**Score:** 3/12 hits

---

| ID | Category | Question | Method | Hit? | Notes |
|---|---|---|---|---|---|
| Q1 | single-fact | What is CT101's role in the GZMO architecture? | qdrant | NO | No expected keywords found in top hit. Top hit: [DECISION:Session] ## GZMO Ecosy... |
| Q2 | single-fact | What is the dual-writer rule? Can two overnight writers run ... | qdrant | NO | No expected keywords found in top hit. Top hit: [DECISION:Session] No dream cycl... |
| Q3 | single-fact | What is the Prime inference server and what port does it run... | qdrant | NO | No expected keywords found in top hit. Top hit: [DECISION:Session] No dream cycl... |
| Q4 | single-fact | What does Obolus do in the AOS energy routing chain? | qdrant | YES | Found keywords: Obolus, IPW, inverse-propensity |
| Q5 | single-fact | What are the stages of the GZMO distillation pipeline? | qdrant | NO | No expected keywords found in top hit. Top hit: [SYSTEM:SessionDistill] Uses syn... |
| Q6 | multi-session | ADR-0003 originally said CT101 is frozen reference, then ADR... | qdrant | NO | No expected keywords found in top hit. Top hit: [DECISION:Session] State: drifti... |
| Q7 | multi-session | ADR-0003 said one writer, ADR-0004 said airgap, ADR-0007 sai... | qdrant | NO | No expected keywords found in top hit. Top hit: [task:deployment smoke test] Use... |
| Q8 | multi-session | How does a TinyFolder drop reach the living vault? Trace the... | qdrant | YES | Found keywords: distill |
| Q9 | multi-session | If a beat-gate passes for one loop, what must happen before ... | qdrant | NO | No expected keywords found in top hit. Top hit: [task:deployment smoke test] Use... |
| Q10 | multi-session | On 2026-07-15, a cutover happened. Was the vault imported fr... | qdrant | NO | No expected keywords found in top hit. Top hit: [DECISION:Session] 9% |
| Phase ... |
| Q11 | single-fact | What are the roles of Qdrant, Neo4j, and SQLite in GZMO? | qdrant | NO | No expected keywords found in top hit. Top hit: [DECISION:Session] ## GZMO Ecosy... |
| Q12 | single-fact | Is the Chaos Engine required for metabolism to function? | qdrant | YES | Found keywords: chaos |

---

## Detailed results

### Q1 — single-fact

**Question:** What is CT101's role in the GZMO architecture?

**Method:** qdrant

**Hit:** NO

**Top hit:** [DECISION:Session] ## GZMO Ecosystem Status — 2026-07-15 05:30 UTC

### Hardware
| Resource | Status |
|---|---|
| CPU | 1.

**Notes:** No expected keywords found in top hit. Top hit: [DECISION:Session] ## GZMO Ecosystem Status — 2026-07-15 05:30 UTC

### Hardware
| Resource | Status...

### Q2 — single-fact

**Question:** What is the dual-writer rule? Can two overnight writers run on the same vault?

**Method:** qdrant

**Hit:** NO

**Top hit:** [DECISION:Session] No dream cycle has run yet — `DREAMS.

**Notes:** No expected keywords found in top hit. Top hit: [DECISION:Session] No dream cycle has run yet — `DREAMS....

### Q3 — single-fact

**Question:** What is the Prime inference server and what port does it run on?

**Method:** qdrant

**Hit:** NO

**Top hit:** [DECISION:Session] No dream cycle has run yet — `DREAMS.

**Notes:** No expected keywords found in top hit. Top hit: [DECISION:Session] No dream cycle has run yet — `DREAMS....

### Q4 — single-fact

**Question:** What does Obolus do in the AOS energy routing chain?

**Method:** qdrant

**Hit:** YES

**Top hit:** [SYSTEM:Obolus] Applies IPW (inverse-propensity weighting) when routing librarian vs local calls

**Notes:** Found keywords: Obolus, IPW, inverse-propensity

### Q5 — single-fact

**Question:** What are the stages of the GZMO distillation pipeline?

**Method:** qdrant

**Hit:** NO

**Top hit:** [SYSTEM:SessionDistill] Uses synthetic sessions/{id}.md paths for distilled session facts

**Notes:** No expected keywords found in top hit. Top hit: [SYSTEM:SessionDistill] Uses synthetic sessions/{id}.md paths for distilled session facts...

### Q6 — multi-session

**Question:** ADR-0003 originally said CT101 is frozen reference, then ADR-0005 amended this. What is the current state of CT101 vs workstation living-host placement?

**Method:** qdrant

**Hit:** NO

**Top hit:** [DECISION:Session] State: drifting, low-tension.

**Notes:** No expected keywords found in top hit. Top hit: [DECISION:Session] State: drifting, low-tension....

### Q7 — multi-session

**Question:** ADR-0003 said one writer, ADR-0004 said airgap, ADR-0007 said no lite SKU. What is the current product story?

**Method:** qdrant

**Hit:** NO

**Top hit:** [task:deployment smoke test] User requested a deployment smoke test in chat session 8f0096d4 on 2026-07-10 21:21 UTC

**Notes:** No expected keywords found in top hit. Top hit: [task:deployment smoke test] User requested a deployment smoke test in chat session 8f0096d4 on 2026...

### Q8 — multi-session

**Question:** How does a TinyFolder drop reach the living vault? Trace the path through Brain Feed.

**Method:** qdrant

**Hit:** YES

**Top hit:** [SYSTEM:SessionDistill] Uses synthetic sessions/{id}.md paths for distilled session facts

**Notes:** Found keywords: distill

### Q9 — multi-session

**Question:** If a beat-gate passes for one loop, what must happen before it lands in the living host?

**Method:** qdrant

**Hit:** NO

**Top hit:** [task:deployment smoke test] User requested a deployment smoke test in chat session 8f0096d4 on 2026-07-10 21:21 UTC

**Notes:** No expected keywords found in top hit. Top hit: [task:deployment smoke test] User requested a deployment smoke test in chat session 8f0096d4 on 2026...

### Q10 — multi-session

**Question:** On 2026-07-15, a cutover happened. Was the vault imported from CT101 or fresh data-next?

**Method:** qdrant

**Hit:** NO

**Top hit:** [DECISION:Session] 9% |
| Phase | Idle |
| Crystallized thoughts | 603 |
| Deaths | 0 |

### Memory & Knowledge
| Component | Status |
|---|---|
| Vault DB | Exists (152 KB) |
| Knowledge Graph | **Em

**Notes:** No expected keywords found in top hit. Top hit: [DECISION:Session] 9% |
| Phase | Idle |
| Crystallized thoughts | 603 |
| Deaths | 0 |

### Memory ...

### Q11 — single-fact

**Question:** What are the roles of Qdrant, Neo4j, and SQLite in GZMO?

**Method:** qdrant

**Hit:** NO

**Top hit:** [DECISION:Session] ## GZMO Ecosystem Status — 2026-07-15 05:30 UTC

### Hardware
| Resource | Status |
|---|---|
| CPU | 1.

**Notes:** No expected keywords found in top hit. Top hit: [DECISION:Session] ## GZMO Ecosystem Status — 2026-07-15 05:30 UTC

### Hardware
| Resource | Status...

### Q12 — single-fact

**Question:** Is the Chaos Engine required for metabolism to function?

**Method:** qdrant

**Hit:** YES

**Top hit:** [DECISION:Session] md` | **Missing** — no dream cycle has executed |
| Synapse events | Directory exists |
| Obolus routing | Directory exists |

### Codebase
| Crate | Purpose |
|---|---|
| `gzmo-cor

**Notes:** Found keywords: chaos

