# MemoryArena Baseline — VERDICT

**Date:** 2026-08-22
**ADR:** [ADR-0008](../../docs/adr/ADR-0008-edge-ssm-memory.md) — Option B
**Paper:** arXiv:2608.13883 (MemoryLake on MemoryArena)

---

## Baseline result

**Score:** 3/12 hits (25%)

The current GZMO memory system (Qdrant honeypot collection backed by SQLite vault FTS) was tested against 12 questions grounded in real GZMO history, spanning single-fact recall and multi-session interdependent tasks.

### What worked (3/12)

- **Q4 (Obolus/IPW):** Single-fact recall hit — the FTS found the `[SYSTEM:Obolus] Applies IPW (inverse-propensity weighting)` fact directly. Keyword overlap was strong.
- **Q8 (TinyFolder → distill):** Partial multi-session hit — found `distill` keyword in a SessionDistill fact, but did not trace the full path (session close → distill queue → honeypot → vault).
- **Q12 (Chaos Engine):** Single-fact hit — found `chaos` keyword in an ecosystem status fact, though the hit was tangential (status table, not the "opt-in" policy).

### What failed (9/12)

- **Q1 (CT101 role):** The top hit was an ecosystem status table, not the `[POLICY:CT101Frozen]` or `[POLICY:ADR0003]` facts that directly answer this question. The FTS matched "CT101" but ranked a generic status snapshot above the policy fact.
- **Q2 (dual-writer rule):** Top hit was "No dream cycle has run yet" — completely irrelevant. The dual-writer rule is in ADR docs, not in the honeypot.
- **Q3 (Prime server):** Same irrelevant top hit. The `[SERVICE:Prime]` fact exists in the honeypot but was not ranked as top hit.
- **Q5 (distillation pipeline stages):** Found SessionDistill but not the full pipeline description (extract → verify → promote → vault → honeypot).
- **Q6 (ADR evolution — CT101 vs workstation):** Multi-session interdependent question. Top hit was "State: drifting, low-tension" — a chaos state, not an ADR. The system cannot trace ADR evolution across documents.
- **Q7 (ADR product story evolution):** Top hit was a deployment smoke test task — completely irrelevant. The system cannot synthesize across ADR-0003/0004/0007 to answer "what is the current product story."
- **Q9 (beat-gate → promote-by-loop chain):** Multi-session interdependent question. Top hit was a deployment smoke test. The system cannot trace the dependency chain (beat-gate PASS → operator ack → mutex claim → PROMOTE_ACK).
- **Q10 (CT101 cutover history):** Top hit was a status table fragment. The `[POLICY:CT101Frozen]` fact about the cutover exists in the honeypot but was not retrieved.
- **Q11 (Qdrant/Neo4j/SQLite roles):** Top hit was the ecosystem status table, not the `[CONCEPT:GZMO]` fact that describes the pipeline architecture.

### Pattern of failure

1. **FTS ranking is poor.** The system returns keyword-matching facts but ranks them by FTS relevance, not semantic relevance. Generic status snapshots outrank specific policy/concept facts.
2. **No cross-document synthesis.** Multi-session questions (Q6, Q7, Q9) require tracing facts across ADRs and session histories. The current system retrieves isolated facts, not chains.
3. **No temporal awareness.** Questions about ADR evolution (Q6, Q7) require knowing that ADR-0003 was amended by ADR-0005, which was then superseded by ADR-0007. The honeypot stores facts but not their supersession relationships.
4. **Relevant facts exist but are not retrieved.** The `[POLICY:CT101Frozen]`, `[SERVICE:Prime]`, and `[CONCEPT:GZMO]` facts are in the honeypot, but the FTS query surfaces wrong results for their questions.

## Comparison with MemoryArena expectations

MemoryArena (arXiv:2608.13883) evaluates memory systems on interdependent, multi-session task completion — not post-hoc recall. The current GZMO system scores:
- **Single-fact recall:** 2/5 (40%) — decent for direct keyword matches, poor for semantic ranking.
- **Multi-session interdependent:** 1/7 (14%) — near-failure. The system cannot trace dependency chains across sessions.

MemoryLake's reported suite-level SR is 20.5% (vs 13.6% for the best comparator). The current GZMO system at 25% overall is comparable on raw recall, but the interdependent-task score (14%) is the weakness that Option B targets.

## Recommendation: **HOLD** (pending migration path verification)

The baseline demonstrates a real weakness: the current system handles single-fact recall adequately (when keywords match) but fails on multi-session interdependent tasks. This justifies investigating Option B.

However, GO is **HOLD** pending:

1. **MemoryLake backend self-hostability.** The benchmark code is Apache-2.0 on GitHub, but the MemoryLake backend itself (PyPI `memorylake`, Powerdrill) may be a hosted SaaS. Must verify it can run airgapped (ADR-0004).
2. **Migration path.** Any adoption must preserve ADR-0003 (single-writer) and ADR-0004 (airgap). Must not break the overnight metabolism pipeline. This is a vault-schema migration: atomar step with rollback, behind living-host mutex.
3. **Cost/benefit.** The current system's failures are in FTS ranking and cross-document synthesis — some of which could be addressed with hybrid search (Qdrant prefetch/RRF + cross-encoder rerank, already GATED as C2 in the backlog) without adopting MemoryLake's full multi-track backend.

**If MemoryLake is self-hostable and airgap-compatible, and the migration path is verified, this moves to GO.**

---

*Verdict: GZMO operator surface (OpenClaw) · 2026-08-22 · no runtime code changed*
