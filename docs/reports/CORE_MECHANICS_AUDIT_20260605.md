# Core Mechanics Logic Audit — 2026-06-05 (rerun)

**Repo:** `/home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO`  
**Baseline commit:** `cc58c5d` — *Add provenance-linked tiered memory for honest strict recall.*  
**Plan:** [`core_mechanics_audit_2c2c9c91.plan.md`](../../../.cursor/plans/core_mechanics_audit_2c2c9c91.plan.md) (full entanglement scope; supersedes narrow audit)  
**Method:** Code reading at `cc58c5d` + live working tree + SQLite invariant queries on `data/vault.db`. No production mutation. Strict recall / faithfulness not re-run (requires Prime + VM200); used recorded 2026-06-05 artifacts.

**Audit layers:**

| Layer | What it represents |
|-------|-------------------|
| **Baseline (`cc58c5d`)** | Tiered-memory landing — flaws as shipped |
| **Working tree** | Uncommitted post-audit fixes (29 tracked files modified) |
| **Live DB** | Pre-fix ingest data — F1 localization fix does **not** retroactively repair rows |

---

## Executive summary

- **Invariants checked (Phase A):** 12. All store-correctness invariants **PASS** on live `data/vault.db`.
- **Baseline flaws (F1–F10):** **8 CONFIRMED**, **1 REFUTED** (F13 relation case filter), **1 PARTIAL** (F9).
- **Working-tree fix status:** **7 of 10 baseline flaws have code/eval/doc fixes** in the uncommitted tree; **live store and judge metrics unchanged** until re-ingest + fresh eval run.
- **Blocking for Phase D recertify:** **Yes** — `faithfulness_context` 0.806 < gate 0.90; live evidence rows still reflect entity-level clone (F1 at ingest time).
- **Core honesty finding (unchanged):** The tiered store is structurally sound, but **eval strict success on stale Tier-2 rows overstates grounding** until wave re-ingest. Working-tree closes the eval/prod gap (F5) and ingest-path flaws (F1, F4, F6) in code — not yet in data.

### Metrics snapshot (this session)

| Metric | Value | Source |
|--------|-------|--------|
| `user_version` | **5** (v6 `distill_dedup` in code, not yet migrated on live DB) | `PRAGMA user_version` |
| `semantic_vault` | 5458 | live DB |
| `honeypot` (all / `is_latest=1`) | 2093 / 2002 | live DB |
| `honeypot_fts` (latest / stale superseded) | 2002 / **91** | live DB — rebuild on next `recall_rrf` (fixed guard in WIP) |
| `evidence` / `evidence_fts` | 1010 / 1010 | live DB |
| orphan evidence | **0** | live DB |
| evidence on superseded facts | **77** | live DB — unreachable via recall |
| honeypot latest with evidence | 933 / 2002 (**46.6%**) | live DB — expected <1.0; not all facts localize |
| `recall_at_5_rrf_strict` (full) | **38/87 (0.437)** | `recall-metrics.json` 2026-06-05T10:36:57 |
| `faithfulness_context` | **0.806** (29/36) | `faithfulness-judge-latest.json` |
| golden files honeypot-excluded | **17/50 (34%)** | `expected.yaml` scan |
| golden probes excluded | **16** on excluded files, **33** eligible | `expected.yaml` |

---

## Entanglement findings (high-risk edges)

| Edge | Baseline `cc58c5d` | Working tree + live DB |
|------|-------------------|------------------------|
| verify → evidence (Tier-2 populated) | **PASS** | **PASS** — 1010 rows, all `verify_pass=1`, all embedded |
| evidence.fact_id → honeypot.id | **PASS** | **PASS** — 0 orphans; strict 1:1 `evidence.id == fact_id` |
| `qualifies_for_honeypot` → golden scope | **FAIL** (F3) | **FAIL** — 17/50 files structurally unreachable |
| strict eval uses evidence_text; scratch uses Tier-1 | **FAIL** (F5) | **FIXED in code** — `source_span:` in scratch inject; **live agent path needs rebuild+deploy** |
| Qdrant mirrors honeypot only, not evidence | **PASS** (by design) | **PASS** — `sync-vault-to-qdrant.py --source honeypot` |
| ingest-eval dry-run → report.json | **FAIL** (F2) | **FAIL** (by design) — documented in certify + README |
| entity-level evidence → N observations | **FAIL** (F1) | **FIXED in code** (`localize_observation_evidence`); **live DB still entity-cloned rows** |
| reranker sees Tier-1 only | **FAIL** (F4) | **FIXED in code** — evidence appended to rerank doc |
| superseded honeypot rows in FTS | **WARN** (N1) | **FIXED guard** — next recall rebuilds; DB still has 91 stale until then |
| purge → evidence cascade | **FAIL** (N2) | **FIXED** — `DELETE FROM evidence WHERE source_file LIKE …` in purge script |
| certify ingest contract vs live store | **FAIL** (F8) | **PARTIAL FIX** — labeled "EXTRACTION SNAPSHOT"; strict floor 0.35 gating added |
| session_distill → honeypot | **FAIL** (F6) | **FIXED in code** — synthetic `sessions/<id>.md` source_file |
| `anchor_decay_classes` SessionDistill vs honeypot | **FAIL** (F14) | **FIXED** once F6 lands + distill runs |

---

## Phase results

### Phase A — Store invariants: **PASS**

Live queries on `data/vault.db`:

```
user_version 5
orphan_evidence 0
dup_fact_ids 0
evidence_id_eq_factid 1010/1010
evidence no_embed 0
evidence verify_pass!=1 0
honeypot is_latest=1 conf<0.85: 0
char_start NULL 0/1010
relation truths in honeypot: 0
evidence on superseded facts: 77
honeypot_fts stale (is_latest=0): 91
```

| Invariant | Query / check | Result |
|-----------|---------------|--------|
| A.1.1 schema v5+ | `PRAGMA user_version` | **PASS** (5; v6 pending first open with WIP binary) |
| A.1.2 evidence FK | orphan `fact_id` | **PASS** (0) |
| A.1.3 evidence_fts sync | count vs evidence | **PASS** (1010 = 1010) |
| A.1.4 honeypot_fts sync | latest rows indexed | **PASS at query time** (`is_latest=1` join); index dirty (91 stale rows) until next recall |
| A.2.1 evidence ratio | latest honeypot with evidence | **PASS** (46.6% — documented, not 1.0) |
| A.2.2 relation truths | `[RELATION:*]` in honeypot | **PASS** (0) |
| A.2.3 evidence embeddings | NULL/empty | **PASS** (0) |
| A.2.4 1:1 fact_id | duplicate evidence rows | **PASS** (0) |
| A.3.1 quarantine floor | conf < 0.85 in honeypot | **PASS** (0) |
| A.3.2 quarantine in recall | streams join `is_latest=1` | **PASS** (by construction) |
| A.3.3 corroboration evidence | lifecycle path | **PASS** (code review; 77 superseded evidence unreachable) |

### Phase B — Ingest & promotion: **mixed**

**B.1 Live vs dry-run**

| Path | Writes honeypot? | Writes evidence? | Verdict |
|------|------------------|------------------|---------|
| `gzmo ingest` / `ingest-dir` | Yes | Yes | Production recall store |
| `gzmo ingest-eval` | **No** | **No** | `ingest_file_dry_run` + empty `ToolRegistry` (no Neo4j MCP) |

**Verdict:** "Ingest contract PASS" **does not** imply recall store current (F2 **CONFIRMED** at baseline; documented in WIP certify).

**B.2 Extract → verify → truth shaping**

| Step | Baseline `cc58c5d` | Working tree |
|------|-------------------|--------------|
| B.2.1 Verifier ≥12 char quote | **PASS** | **PASS** |
| B.2.2 Tier-1 drops raw archive sentence | **PASS** (by design) | **PASS** |
| B.2.3 Entity-level evidence clone | **FAIL** (F1) — `evidence_span_clone` per entity | **FIXED** — `localize_observation_evidence(body, obs, &ve.evidence, obs_count)` |
| B.2.4 Relations localized | **PASS** | **PASS** |
| B.2.5 `relink_relations` empty evidence | **CONFIRMED** (relations may promote without span) | unchanged |

**B.3 evidence_localize:** **PASS** — 5/5 unit tests including `observation_evidence_per_obs_not_shared_entity_quote`. Live DB: `char_start NULL = 0/1010`.

**B.4 Honeypot eligibility vs golden:** **FAIL** (F3) — 17/50 files match exclusion patterns (`sources`, `quelltext`, `chat_history`, `chat_session`). WIP `run-recall-eval.py` reports separate `honeypot-eligible` vs `excluded-source` tracks.

**B.5 Lifecycle:** Superseded facts retain evidence rows (77) — unreachable, not orphan. Purge path now deletes evidence (N2 fix).

### Phase C — Recall / RRF: **PASS (logic)**

`recall_rrf` stream order verified in `vault.rs`:

1. honeypot FTS → evidence FTS → graph (or keyword fallback) → vector (Qdrant ⨝ local) → evidence vector
2. `rrf_fuse` + `STREAM_TOP_RESCUE` (top-5 per stream)
3. `diversify_by_source_file` (`RERANK_STAGE_PER_FILE=12`)
4. `apply_rerank` → `truncate(limit)`

| Check | Baseline | Working tree |
|-------|----------|--------------|
| Evidence streams boost `fact_id` | **PASS** | **PASS** |
| Rerank sees Tier-2 | **FAIL** (F4) | **FIXED** — `get_evidence_text` appended to rerank doc |
| Rescue equal weight | **PASS** (same formula all streams) | **PASS** |
| `get_evidence_text` LIMIT 1 | **PASS** (1:1 design makes LIMIT moot) | **PASS** |
| `STRICT_MIN_CHARS=8` | **PARTIAL** (F9) | WIP adds `strict_claim_is_substantive()` |

**E3 bucket:** 13 facts in store but not top-5 — per prior triage ([RRF_STRICT_LOST_FACTS_20260604.md](./RRF_STRICT_LOST_FACTS_20260604.md)); F4 fix + F1 re-ingest are primary levers. Per-stream rank logging not re-run this session.

### Phase D — Eval & gate alignment: **FAIL**

**Three-truths cross-matrix:**

| Scenario | Observed | Interpretation |
|----------|----------|----------------|
| strict PASS (38/87), context FAIL (0.806) | **Yes** | Hits carry evidence_text but judge finds unsupported claims — Tier-2 fidelity flaw (F1 data) + paraphrase/context mismatch |
| strict FAIL, context PASS | Shrinking post-tiered | Not dominant in latest run |
| strict PASS, corpus FAIL | Possible | Localization window ≠ full golden sentence |
| ingest contract PASS, strict FAIL | **Common** | F2 + F8 — dry-run snapshot vs live store |

**certify-production-baseline.sh (WIP):**

| Section | Uses live store? | Status |
|---------|------------------|--------|
| Golden audit | YAML only | Low stale risk |
| Ingest contract | Frozen `baseline-m4-post-sprint.json` | **Labeled snapshot** + vault-newer WARN |
| Strict recall | Live honeypot + evidence | Good — **now gated** at `recall_rrf_strict_min: 0.35` |
| Faithfulness judge | `gzmo_report.json` hits | **FAIL** — 0.806 < 0.90 |

**mem-score:** Uses same-session `recall-metrics.json` + judge report when `--merge-mem-score` run together; no mixed-timestamp issue detected in artifacts.

### Phase E — Runtime consumers: **FIXED in code / gap in live deploy**

| Consumer | Reads | Uses evidence_text? | Baseline | Working tree |
|----------|-------|---------------------|----------|--------------|
| Eval (`memory search --json`) | `MemoryHit` | **Yes** | PASS | PASS |
| Chat scratch `[RECALL]` | `RecallSnippet` | **No** | **FAIL** (F5) | **FIXED** — `source_span:` + `fact_id` |
| `memory_search_core` text | Tier-1 content | No | informational | unchanged |
| Spark / Dream REM | Tier-1 anchors | No | by design | by design |
| Session distill | transcript → vault | No evidence at baseline | **FAIL** (F6) | **FIXED** — synthetic source_file |
| Profile | honeypot.content | No | by design | by design |

**Core finding:** At `cc58c5d`, 38/87 strict measured a payload the agent never saw. WIP closes this in scratch inject; **rebuild required** before production reflects fix.

### Phase F — Sync & ops: **PASS with fixes pending activation**

| Check | Result |
|-------|--------|
| F.1 Qdrant = honeypot only (`is_latest=1`) | **PASS** |
| F.1.2 evidence vectors local SQLite full-scan | **PASS** (by design) |
| F.1.3 split-brain superseded in Qdrant | Mitigated by `is_latest=1` filter on sync |
| F.2 Neo4j provenance vs SQLite evidence | Not live-queried; write paths diverge by design |
| F.3 purge deletes evidence | **FIXED** in WIP `purge-wave-ingest.sh` |
| N1 FTS stale rows | **FIXED guard** in WIP `ensure_honeypot_fts_synced` — self-heals on next recall |

### Phase G — Flaw register (F1–F10 + extensions)

| ID | Flaw | Baseline `cc58c5d` | Working tree | Fix priority |
|----|------|-------------------|--------------|--------------|
| F1 | Entity-level evidence shared | **CONFIRMED** | **FIXED** (code; **re-ingest required**) | **P0** |
| F2 | ingest-eval dry-run ≠ store | **CONFIRMED** | **CONFIRMED** (documented) | P1 doc |
| F3 | Golden on excluded files | **CONFIRMED** (17/50) | **CONFIRMED**; dual-track eval added | **P0** eval |
| F4 | Rerank Tier-1 only | **CONFIRMED** | **FIXED** | P1 |
| F5 | Scratch ignores evidence | **CONFIRMED** | **FIXED** | **P0** deploy |
| F6 | session_distill no honeypot | **CONFIRMED** | **FIXED** | P1 |
| F7 | Verifier `-1` index parse | **CONFIRMED** | **FIXED** + unit test | P2 |
| F8 | certify stale ingest report | **CONFIRMED** | **PARTIAL** — labeled + WARN | P1 |
| F9 | Strict hollow wins | **PARTIAL** | **FIXED** — substantive heuristic | P2 |
| F10 | Spec missing evidence tier | **CONFIRMED** | **FIXED** — MEMORY_ARCHITECTURE_SPEC §2.3.1 | P2 |
| F12 | `is_unverified_derived` origin mismatch | **CONFIRMED** | **FIXED** — includes `verified_dream`/`session_distill` | P1 |
| F13 | Relation case filter bug | **REFUTED** | **REFUTED** | — |
| F14 | Spark SessionDistill anchors dead | **CONFIRMED** | **FIXED** once F6 runs | P1 |
| N1 | honeypot_fts stale superseded rows | **CONFIRMED** | **FIXED** guard | P1 |
| N2 | purge no evidence delete | **CONFIRMED** | **FIXED** | P1 |
| N3 | strict recall non-gating | **CONFIRMED** | **FIXED** — floor 0.35 in gate-config | P1 |
| N4 | secrets in gzmo.toml | **CONFIRMED** | **FIXED** — keys cleared to `""` | **P0 ops** (rotate if ever committed live) |

---

## Recommended fixes (ordered)

### P0 — unblock honest recertify

1. **Wave re-ingest** after deploying WIP binary — F1 fix does not repair 1010 existing evidence rows.
2. **Re-run** `run-recall-eval.py --match strict` + `faithfulness-judge.py --gate` on fresh store; target `faithfulness_context ≥ 0.90`.
3. **Deploy WIP** to daemon (F5 scratch `source_span`, F4 rerank, F1 ingest path).
4. **F3** — report strict on `honeypot-eligible` track as primary KPI; keep excluded-source informational.

### P1 — consistency

5. Run one `recall_rrf` query to trigger FTS rebuild (N1 self-heal) or explicit FTS maintenance.
6. Purge 77 superseded-fact evidence rows (or wave re-ingest replaces wholesale).
7. **F8** — either refresh `baseline-m4-post-sprint.json` via live replay or remove ingest contract from certify entirely.
8. **F18** — opt-in `LIVE_INGEST_SMOKE=1` in certify to exercise Neo4j MCP write path.

### P2 — hygiene

9. Migrate live DB to schema v6 (`distill_dedup`) on first WIP vault open.
10. **F7** NotebookLM file re-ingest after verifier fix.
11. Sync INFRASTRUCTURE_OVERVIEW to `enabled = true` reality (N5).

---

## Success criteria vs plan §11

| Criterion | Status |
|-----------|--------|
| Every entanglement box has PASS/FAIL | **Yes** — table above |
| F1–F10 each CONFIRMED/REFUTED | **Yes** |
| Three-truths cross-matrix filled | **Yes** — Phase D |
| E3 per-stream ranks | **Deferred** — prior doc; not re-run (needs instrumented recall) |
| Consumer gap documented | **Yes** — Phase E |
| Fix backlog P0/P1/P2 | **Yes** |

---

## What this audit did not cover

- Prime LLM extraction quality (only verify→store contract)
- Neo4j entity-linking accuracy (live Neo4j not queried)
- Performance/latency tuning
- Full auth/security review (N4 flagged)
- Spark hypothesis algorithm beyond honeypot entanglement
- Live strict/faithfulness re-run (Prime + VM200 required)

---

## Reference

- Plan: [`core_mechanics_audit_2c2c9c91.plan.md`](../../../.cursor/plans/core_mechanics_audit_2c2c9c91.plan.md)
- Prior findings: [RRF_STRICT_LOST_FACTS_20260604.md](./RRF_STRICT_LOST_FACTS_20260604.md), [ANTIGRAVITY_TIERED_MEMORY_RESULTS_20260605.md](./ANTIGRAVITY_TIERED_MEMORY_RESULTS_20260605.md)
- Key code: `vault.rs`, `honeypot.rs`, `ingest.rs`, `evidence_localize.rs`, `platform_memory.rs`, `scratch.rs`, `session_distill.rs`
- Key gates: `certify-production-baseline.sh`, `run-recall-eval.py`, `gate-config.yaml`, `purge-wave-ingest.sh`
