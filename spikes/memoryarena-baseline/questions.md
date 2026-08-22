# MemoryArena Baseline — Questions

Grounded in real GZMO history from `data-next/vault.db`, `docs/`, and `data-next/` artifacts.
Questions span (a) single-fact recall and (b) multi-session interdependent tasks.

---

## Single-fact recall

### Q1 — CT101 role
What is CT101's role in the GZMO architecture? (Is it the living host, a frozen reference, or a lab machine?)

### Q2 — Dual-writer rule
What is the dual-writer rule? Can two overnight writers run on the same vault?

### Q3 — Prime server
What is the Prime inference server and what port does it run on?

### Q4 — AOS energy routing
What does Obolus do in the AOS energy routing chain? (How does it handle IPW?)

### Q5 — Honeypot pipeline
What are the stages of the GZMO distillation pipeline? (extract → verify → promote → vault → honeypot?)

## Multi-session interdependent tasks

### Q6 — ADR evolution
ADR-0003 originally said "the workstation is the sole living instance; CT101 is a frozen reference."
Then ADR-0005 amended this. What is the current state of CT101 vs workstation living-host placement?

### Q7 — Single-writer + airgap history
GZMO decided on "one overnight writer per vault" (ADR-0003) and "airgap honesty" (ADR-0004).
Then ADR-0007 said "no lite SKU." What is the current product story — is there a lite product, or just one living Keep?

### Q8 — Brain Feed + TinyFolder chain
TinyFolder is described as a "Brain Feed satellite." It feeds into the living host via overnight metabolism.
Given that ADR-0005 introduced "promote-by-loop," trace the path: how does a TinyFolder drop reach the living vault?
(What intermediate steps happen: session close → distill queue → honeypot → vault?)

### Q9 — Beat-gate + promote-by-loop interdependency
Beat-gates are the "honesty layer" before trusting a composed runtime. ADR-0005 allowed promote-by-loop.
If a beat-gate passes for one loop (e.g., knowledge), what must happen before it lands in the living host?
(What are the gates: beat-gate PASS, operator ack, living-host-mutex claim, PROMOTE_ACK=1?)

### Q10 — CT101 cutover history
On 2026-07-15, a cutover happened. What was the decision? Did the vault get imported from CT101's 60k-fact store, or was it a fresh data-next?

### Q11 — Qdrant vs Neo4j vs SQLite roles
GZMO uses Qdrant (:6333), Neo4j (:7687), and SQLite (vault.db). What is the role of each? (vector search, knowledge graph, source of truth?)

### Q12 — Chaos engine dependency
Is the Chaos Engine required for metabolism to function? (Is it opt-in for chat, or does metabolism depend on it?)
