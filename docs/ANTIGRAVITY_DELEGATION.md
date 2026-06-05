# Antigravity — Delegation queue (post M2)

**Repo:** `/home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO`  
**Updated:** 2026-06-04  
**Active engineering (Cursor):** MemScore P0 ([MEMSCORE_GATES.md](./MEMSCORE_GATES.md), [SOTA_RESEARCH_202606.md](./SOTA_RESEARCH_202606.md)), M3 cognition next.  
**Antigravity — MEGA IX:** probe tune **39.78%** golden track done; optional `RECALL5_TREND_MEGA9.md` + token eval wrap-up.  
**Antigravity — MEGA VIII ✅:** [ANTIGRAVITY_MEGA_VIII_RESULTS.md](./ANTIGRAVITY_MEGA_VIII_RESULTS.md) — 25.81% gzmo vs 24.73% qdrant.  
**Antigravity — MEGA VII ✅ GREEN:** [ANTIGRAVITY_MEGA_VII_RESULTS.md](./ANTIGRAVITY_MEGA_VII_RESULTS.md).  
**Antigravity — optional:** [ANTIGRAVITY_MEGA_VI.md](./ANTIGRAVITY_MEGA_VI.md) (recall pilot / knowledge delete) — superseded for momentum by MEGA VII; run VI phases only if Max re-authorizes keywords.  
**Antigravity — MEGA V ✅:** [MEGA V results](./ANTIGRAVITY_MEGA_V_RESULTS.md).  
**Scratch memory I (done):** [SCRATCH_MEMORY_VERIFY.md](./SCRATCH_MEMORY_VERIFY.md).  
**Scratch memory II:** [SCRATCH_HANDOVER_II_STATUS.md](./SCRATCH_HANDOVER_II_STATUS.md) — **CLOSED GREEN**.  
**Legacy MEGA:** [ANTIGRAVITY_HANDOVER.md](./ANTIGRAVITY_HANDOVER.md) — frozen unless `RUN BATCH2 EVAL`.  
**Legacy:** [**`ANTIGRAVITY_TODO.md`**](./ANTIGRAVITY_TODO.md) (S1–S6 + end-gate ✅ 2026-06-03).  
**Cursor status:** `baseline-m4-platform-20260604` — Antigravity + Cursor aligned GREEN; sign-off [MEGA5_BASELINE_SIGNOFF.md](./MEGA5_BASELINE_SIGNOFF.md).  

**Active (M4 golden content):** [**ANTIGRAVITY_M4_GOLDEN_STEP_BY_STEP.md**](./ANTIGRAVITY_M4_GOLDEN_STEP_BY_STEP.md) — mega guide for `expected.yaml` corpus/context alignment (≤10 probes/session). Contract: [M4_GOLDEN_CONTRACT.md](./M4_GOLDEN_CONTRACT.md).

**Active (Tiered memory / foundational ingest-recall):** [**ANTIGRAVITY_TIERED_MEMORY_STEP_BY_STEP.md**](./ANTIGRAVITY_TIERED_MEMORY_STEP_BY_STEP.md) — provenance-linked two-tier memory (evidence table + recall stream + live re-ingest). **Highest priority** for strict recall M0 (31/87). Diagnosis: [RRF_STRICT_LOST_FACTS_20260604.md](./RRF_STRICT_LOST_FACTS_20260604.md).

Phase 1 (N1–N6) complete → [`ANTIGRAVITY_M4_PHASE1.md`](./ANTIGRAVITY_M4_PHASE1.md).  
Use **one session per handoff**. Log results in `walkthrough.md` + tick boxes below (legacy packages A–E archived).

---

## Package A — Schritt 3 (highest value, ~40–60 min)

**Goal:** Fresh `report.json` after M2 + code fixes; document strict vs layered gate.

```bash
cd /home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO
unset CARGO_TARGET_DIR
cargo build --release -p gzmo-cli -p gzmo-core
scripts/ingest-quality/replay-wave.sh 2>&1 | tee scripts/ingest-quality/replay-wave-post-m2.log
cp scripts/ingest-quality/report.json scripts/ingest-quality/reports/baseline-post-m2.json
GATE_MODE=strict scripts/ingest-quality/gate-report.sh | tee scripts/ingest-quality/gate-post-m2-strict.log
scripts/ingest-quality/gate-report.sh | tee scripts/ingest-quality/gate-post-m2-layered.log
scripts/ingest-quality/check-contract.sh
```

**Deliverables:**

- [ ] Log path + exit codes (strict / layered)
- [ ] Table: zero_ent, rich_few, rel_prom, zero_rel, golden entity %
- [ ] If strict fails only on rel_prom ~0.1pp: note in `docs/M2_HONEYPOT_REPORT.md` (not a blocker)
- [ ] Append summary to `walkthrough.md`

**Do not:** purge, re-ingest live corpus, or delete Qdrant `knowledge` without explicit human OK.

---

## Package B — M2 ops hygiene (~20 min)

**Goal:** Document cutover state for humans.

1. Count comparison:
   ```bash
   curl -s http://192.168.31.202:6333/collections/knowledge | jq .result.points_count
   curl -s http://192.168.31.202:6333/collections/honeypot | jq .result.points_count
   sqlite3 data/vault.db "SELECT COUNT(*) FROM semantic_vault; SELECT COUNT(*) FROM honeypot;"
   ```
2. Update `docs/M2_HONEYPOT_REPORT.md` § “Cutover decision” with:
   - Keep `knowledge` as legacy read-only until M3 RAG paths confirmed
   - OR recommend delete plan (list risks)
3. Verify `scripts/purge-wave-ingest.sh --dry-run wave_01_gzmo_obolus` prints honeypot counts

**Deliverables:**

- [ ] Point counts in walkthrough
- [ ] One-paragraph cutover recommendation for Max

---

## Package C — M3 planning only (no Dream code yet, ~30 min)

**Goal:** Implementation plan for cognition on honeypot (read-only analysis).

Read:

- `docs/MEMORY_ARCHITECTURE_SPEC.md` § M3, `docs/CEILING_ROADMAP.md` § M3
- `gzmo-core/src/dreams.rs`, `gzmo-core/src/spark.rs`, `gzmo-core/src/memory/vault.rs` (`spark_recent_pool`)

Produce in Antigravity brain (or `docs/M3_COGNITION_PLAN.md` if allowed to write):

1. Which functions today read `semantic_vault` / episodic md
2. Minimal diff: switch Qdrant + pool queries to honeypot
3. Test plan: Synapse events should show `source=honeypot` after dream cycle
4. Risks: empty honeypot pool, spark starvation

**Do not:** implement M3 in this package unless human says “M3 implement”.

---

## Package D — Honeypot FTS fix (optional, ~45 min)

**Context:** Broken `trg_honeypot_*` triggers caused SQL logic errors; backfill drops them. FTS table exists but may be stale.

**Goal:** Either:

- **D1:** Rebuild `honeypot_fts` from `honeypot` content (one-shot SQL/Python), no triggers until rowid strategy is defined, OR  
- **D2:** FTS5 `content='honeypot'` external content + correct triggers (see SQLite FTS5 docs)

**Deliverables:**

- [ ] `SELECT COUNT(*) FROM honeypot_fts` matches honeypot row count (approx)
- [ ] Document approach in `M2_HONEYPOT_REPORT.md` § FTS

---

## Package E — Regression sweep (fast, ~5 min)

Run after any Antigravity code change:

```bash
cargo test -p gzmo-core honeypot ingest_prep
cargo test -p gzmo-cli eval_match_tests
scripts/ingest-quality/check-contract.sh
python3 scripts/ingest-quality/retrieval-probes.py
```

---

## FULLGAS mode (~50% budget)

Use **`docs/ANTIGRAVITY_FULLGAS.md`** — ordered A0→A→E→B→C→D→F with checkpoints. User reports `STOPPED_AT` to Cursor when agent dies.

## Suggested order (2026-06-03)

Post-M3 ✅ · Session A ✅ — **`ANTIGRAVITY_M4_PHASE1.md`** § *no collisions*:

1. **M4 Phase 1:** ✅ `M4_PHASE1_RESULTS.md`  
2. **FULLGAS M4 (now):** [`ANTIGRAVITY_FULLGAS_M4.md`](./ANTIGRAVITY_FULLGAS_M4.md) — Opus budget, G0→G9  

**Cursor lane (parallel OK):** Rust spark datetime fix done; may tune `gzmo.toml` / Synapse — **not** `expected.yaml` or `mem-score.py`.

Legacy packages A–D below are **done** or superseded.

---

## Copy-paste master prompt

```
Repo: /home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO
Read docs/ANTIGRAVITY_HANDOVER.md (2026-06-04). Execute H1 then H2. Log walkthrough.md.
FORBIDDEN: Rust, gzmo.toml, purge, ingest-dir, delete Qdrant knowledge, replay-wave unless RUN END GATE.
```

Legacy (pre-2026-06-04):

```
Read docs/ANTIGRAVITY_DELEGATION.md — packages A–E archived; use HANDOVER instead.
```

---

## What Cursor keeps (do not duplicate on Antigravity)

- **`recall_rrf`** + `PlatformMemory::memory_search` hook ([RUST_RECALL_FOLLOWUP_SPEC.md](./RUST_RECALL_FOLLOWUP_SPEC.md))
- Graph lifecycle (`update`/`extends`/`derives`), `gzmo profile` API (post-RRF)
- Small Rust fixes after ingest/eval failures
- M3 honeypot cognition hardening once RRF pilot is green
- PR/commit workflow if requested

**Antigravity next (when Max authorizes):** `RUN KNOWLEDGE DELETE` on/after **2026-06-11** → [M4_KNOWLEDGE_DELETE_RUNBOOK.md](./M4_KNOWLEDGE_DELETE_RUNBOOK.md); optional `RUN RECALL RESCUE PILOT` from [ANTIGRAVITY_MEGA_VI.md](./ANTIGRAVITY_MEGA_VI.md).
