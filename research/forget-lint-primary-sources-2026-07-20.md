# Research brief: `forget-lint` (purposeful forgetting / intermediate-honeypot clearer)

**Date:** 2026-07-20  
**Scope:** Primary-source dig for a future little-tool CLI; no scaffold in this pass.  
**Vaults:** GZMO monorepo `/home/gzmo/github-clone/GZMO`, sibling `/home/gzmo/github-clone/honeypot-gate`.  
**CT101:** read-only SSH evidence only — never import into `data-next` ([docs/UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md)).  
**Related amp:** [tool-dev-amp-forget-verify-token-2026-07-20.md](./tool-dev-amp-forget-verify-token-2026-07-20.md) §1 (shorter sibling).  
**Parent dig:** [ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md).

---

## 1. Problem statement

CT101’s curated honeypot already **creates** intermediate layers via lifecycle supersession (`is_latest=0`) and Extends (both stay latest), then **passively** decays recall scores by half-life. Nothing ships **active forgetting as a product surface** — archaeology names this gap `forget-lint` ([research/ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md) §Executive picks #1).

### CT101 fact evidence (SSH, 2026-07-20)

Read-only: `ssh ct101 "sqlite3 /opt/gzmo/data/vault.db …"` against table `honeypot` (columns verified via `PRAGMA table_info(honeypot)`).

| Id prefix | Full id | `is_latest` | `origin` | `decay_class` | `source_file` | Content |
|-----------|---------|-------------|----------|---------------|---------------|---------|
| `690cb295` | `690cb295-a0a3-459c-ac21-e0a46d7c3553` | 1 | ingest | CuratedVault | `the-cascading-honeypot-theorem-of-wisdom.md` | `[CONCEPT:Purposeful Forgetting] A systematic, biological, and psychological necessity for the human brain.` |
| `26ad76d0` | `26ad76d0-ff95-4b96-b287-d25e988335ec` | 1 | ingest | CuratedVault | `the-cascading-honeypot-theorem-of-wisdom.md` | `[CONCEPT:Purposeful Forgetting] Is a vital, active process that clears intermediate honeypots.` |
| `20811f83` | `20811f83-db7e-4c4a-a3f0-cbbcb65baf3c` | 1 | ingest | CuratedVault | `the-honeypot-compiler-architecture-distilling-dat.md` | `[CONCEPT:Lint/Maintenance] Acts as a mechanism for 'purposeful forgetting' by resolving contradictions.` |
| `f8006497` | `f8006497-e3d1-4329-9d93-6a24cd4b68f6` | 1 | ingest | CuratedVault | `the-cascading-honeypot-theorem-of-wisdom.md` | `[CONCEPT:Systems thinking] Suggests forgetting is a vital process for clearing intermediate honeypots.` |

All four also exist in `semantic_vault` (count=4); none in `quarantine_vault` (count=0 for these ids).

### Scale of the intermediate-layer problem (CT101 census)

| Metric | Value | Source |
|--------|------:|--------|
| honeypot `is_latest=1` | 38 730 | archaeology snapshot + SSH `GROUP BY is_latest` |
| honeypot `is_latest=0` | 9 487 | SSH |
| of which have `supersedes_id` set on the superseded row | 4 026 | SSH (`is_latest=0 AND supersedes_id IS NOT NULL`) — note: successor usually stores `supersedes_id` pointing **at** old id; 4 026 is rows that themselves carry a supersedes pointer while non-latest |
| `quarantine_vault` | 1 012 | archaeology + SSH |
| `honeypot_review_queue` | 4 | SSH (all `reason="relation_row"`, `reviewed=0`) |
| avg age days (`promoted_at`) latest=0 / latest=1 | 40.3 / 38.7 | SSH |
| Decay mix among `is_latest=0` | CuratedVault 9297 · SessionDistill 188 · Structural 2 | SSH |

**Product reading of the four facts:**

1. **Purposeful Forgetting is active, not only half-life** (`690cb295`, `26ad76d0`) — contrasts with today’s decay-only recall path (`SqliteVault::search_with_decay` × `half_life_from_decay_class`).
2. **Lint/Maintenance = contradiction resolution as forgetting** (`20811f83`) — maps to `LifecycleKind::Contradicts` + ripen Phase 2, not to qualification gates.
3. **Clear intermediate honeypots** (`f8006497`, `26ad76d0`) — the operational target is non-latest / superseded / stale intermediate rows, not wholesale vault wipe.

**Catalog gap:** `honeypot-gate` qualifies and classifies **before/around** promote; it does not clear intermediate layers ([honeypot-gate/CONTEXT.md](../../honeypot-gate/CONTEXT.md), archaeology “vs catalog”). Spec already named an MCP surface `gzmo_memory_forget` = soft-delete / `is_latest=0` + audit ([docs/MEMORY_ARCHITECTURE_SPEC.md](../docs/MEMORY_ARCHITECTURE_SPEC.md) §9) — **not implemented** in `gzmo-core/src/mcp/` (grep: no `memory_forget`).

---

## 2. Primary-source inventory

### 2.1 GZMO core — memory plane

| Path | Key symbols | Role for forget-lint |
|------|-------------|----------------------|
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/memory/ripen.rs` | `RipenConfig`, `ripen_honeypot`, `group_by_entity`, `resolve_contradictions`, `export_cards`, `ConceptCard`, `EntityEntry`, `extract_entity_label`, `synthesize_summary` | M5 reads **all** honeypot rows with `confidence >= min_confidence` (including `is_latest=0`); contradiction flag = `entries.iter().any(|e| !e.is_latest)`; winner sort `confidence * (1 + recall_count)` |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/memory/honeypot.rs` | `HONEYPOT_MIN_CONFIDENCE` (0.85), `qualifies_for_honeypot`, `insert_honeypot_lifecycle`, `upsert_honeypot_row`, `sync_honeypot_fts_row` | Ingress write path; FTS sync on insert/upsert; **no delete/forget API** |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/memory/lifecycle.rs` | `LifecycleKind`, `classify_truth_pair`, `extract_primary_entity`, `contradicts_heuristic`, `is_extension`, `is_unverified_derived`, `find_latest_honeypot_by_entity`, `supersede_honeypot` | Only mutator that sets `is_latest=0` today |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/types.rs` | `DecayClass`, `DecayClass::half_life_days`, `SemanticFact.half_life_days`, `ExtractedTruth` | Canonical half-lives for typed enum (Episodic 30 … Structural ∞) |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/memory/vault.rs` | `SqliteVault::store`, `list_quarantine`, `half_life_from_decay_class`, `search_with_decay`, `ensure_honeypot_fts_synced`, `promote_truths_with_origin`, `promote_new_vault_truth`, `promote_corroborate_vault`, `promote_mature_to_honeypot`, `get_memory_chain`, `archive_stale_session_anchors`, `count_honeypot_latest`, schema for `honeypot` / `quarantine_vault` | Quarantine barrier, decay scoring, FTS purge of non-latest, promote lifecycle wiring, nearest existing “archive” helper |
| `/home/gzmo/github-clone/GZMO/gzmo-cli/src/promote_cmd.rs` | calls `promote_mature_to_honeypot` | Overnight mature promote — inverse direction of forget |
| `/home/gzmo/github-clone/GZMO/gzmo-cli/src/daemon_cmd.rs` | `archive_stale_session_anchors` | Soft demote of `[Session …]` vault stubs — pattern adjacent to forget, different target |
| `/home/gzmo/github-clone/GZMO/docs/MEMORY_ARCHITECTURE_SPEC.md` | §3 lifecycle target, §9 `gzmo_memory_forget` | Spec intent for soft-delete + audit |
| `/home/gzmo/github-clone/GZMO/docs/INFRASTRUCTURE_MAP.md` | entanglement row “Qdrant upsert without supersede delete” | Vector mirror retains stale points after SQLite supersede |
| `/home/gzmo/github-clone/GZMO/docs/CORE_MECHANICS_AUDIT_20260605.md` | `honeypot_fts stale (is_latest=0): 91` | Historical FTS dirtiness until recall sync |
| `/home/gzmo/github-clone/GZMO/docs/ct101-systems/50-memory-data-plane/lifecycle-ripen.md` | operator doc for lifecycle + ripen | Cron `honeypot_ripen`; confirms supersede during promote |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/spark.rs` | `stale_sweetness`, spark anchor stale window | **Do not confuse** with forget: spark *reuses* stale anchors; forget *clears* intermediate layers |
| `/home/gzmo/github-clone/GZMO/scripts/ripen-knowledge-core.{sh,py}` | ops wrappers | Living ripen path — out of piece scope |

### 2.2 honeypot-gate (sibling piece)

| Path | Key symbols | Role |
|------|-------------|------|
| `/home/gzmo/github-clone/honeypot-gate/CONTEXT.md` | Stage-2 cognition-smoke | Operator intent: qualify → `gate-report.json` |
| `/home/gzmo/github-clone/honeypot-gate/src/qualify.rs` | `qualifies_for_honeypot`, `rejection_reasons`, `is_boilerplate`, `is_unverified_derived`, `HONEYPOT_MIN_CONFIDENCE` | Port of gzmo qualify; **ingress only** |
| `/home/gzmo/github-clone/honeypot-gate/src/lifecycle.rs` | `LifecycleKind`, `LifecycleAction`, `classify_truth_pair`, `classify_truth_pair_with_action`, `LifecycleAction::{KeepBothAsLatest, SupersedeOldInsertUpdate, …}` | Classification + **declared** storage action; **no DB writes** |
| `/home/gzmo/github-clone/honeypot-gate/src/audit.rs` | `audit_vault` | Read-only vault scan → qualify each honeypot row (includes `is_latest` in `AuditRow`) |
| `/home/gzmo/github-clone/honeypot-gate/src/main.rs` | `check`, `classify`, `audit`, `pick-query` | CLI surface — no plan/apply forget |
| `/home/gzmo/github-clone/honeypot-gate/src/types.rs` | `DecayClass` (includes Core/Semantic/Procedural), `GateReport`, `AuditReport` | Broader decay enum than gzmo `types.rs` |
| `/home/gzmo/github-clone/honeypot-gate/fixtures/sample-facts.jsonl` | 6 fixture truths | Gate smoke only |
| `/home/gzmo/github-clone/honeypot-gate/README.md` | piece card | “No LLM — rule-based gates only” |

### 2.3 CT101 live schema notes (SSH)

`honeypot` columns (order): `id`, `vault_id`, `content`, `content_norm`, `embedding`, `origin`, `memory_type`, `graph_rel`, `supersedes_id`, `is_latest`, `verify_pass`, `confidence`, `decay_class`, `source_file`, `container_tag`, `promoted_at`, `last_recalled_at`, `recall_count`.

`quarantine_vault`: `id`, `content`, `embedding`, `half_life_days`, `confidence`, `created_at`.

`honeypot_review_queue`: `vault_id`, `reason`, `content_preview`, `confidence`, `source_file`, `queued_at`, `reviewed`.

---

## 3. Algorithm / data-model detail (candidate selection)

### 3.1 How intermediate layers are created today

From `SqliteVault::promote_new_vault_truth` ([vault.rs](../gzmo-core/src/memory/vault/promote.rs)):

| `LifecycleKind` | Storage effect | Intermediate residue |
|-----------------|----------------|----------------------|
| `Duplicate` | corroborate existing vault/honeypot | none new |
| `Contradicts` | `supersede_honeypot(old)` → `UPDATE … is_latest=0`; insert new with `graph_rel=update`, `supersedes_id=old` | **superseded row remains** (`is_latest=0`) |
| `Extends` | insert new with `graph_rel=extends`, `supersedes_id=old`; **old stays `is_latest=1`** | **both latest** — enrichment chain, not tombstoned |
| `Unrelated` / `Derives` | fall through to independent insert (Derives blocked from honeypot via `is_unverified_derived` at qualify time) | normal latest row |

`supersede_honeypot` ([lifecycle.rs](../gzmo-core/src/memory/lifecycle.rs)):

```sql
UPDATE honeypot SET is_latest = 0
 WHERE (id = ?1 OR vault_id = ?1) AND is_latest = 1
```

Recall/FTS/Qdrant streams default to `is_latest = 1` (`search_with_decay`, FTS joins, `count_honeypot_latest`). History via `get_memory_chain` walks `supersedes_id`.

### 3.2 Passive decay (not forgetting)

`DecayClass::half_life_days` ([types.rs](../gzmo-core/src/types.rs)):

| Class | Half-life days |
|-------|---------------:|
| Episodic | 30 |
| CuratedVault / SessionDistill | 60 |
| FlexibleIdentity | 139 |
| AbsoluteIdentity | 693 |
| Structural | ∞ |

Vault runtime map `half_life_from_decay_class` ([vault.rs](../gzmo-core/src/memory/vault/mod.rs)) **extends** the enum with string labels seen in live DB: `Core`→36500, `Semantic`→365, `Procedural`→90, default 60. Score:

```text
effective_days = max(0, days_since_access - confirmation_or_recall_count * 5)
decay_multiplier = 0.5 ^ (effective_days / half_life)
```

Facts age out of ranking; **rows are not removed**.

### 3.3 Quarantine

Barrier: `confidence < 0.85` → `INSERT OR REPLACE INTO quarantine_vault` (`store`, `promote_truths_with_origin`). List via `list_quarantine`. CT101 sample rows are spark-style `[HYPOTHESIS …]` at confidence 0.6. Separate: `honeypot_review_queue` for relation-row HITL (4 open).

### 3.4 Ripen M5 interaction (important for candidate design)

Documented phases in module rustdoc vs **actual** `ripen_honeypot` body:

| Doc claim | Code reality |
|-----------|--------------|
| Phase 1: global dedup by `content_norm` similarity > `dedup_threshold` (0.95) | **`dedup_threshold` is unused**; Phase 1 is `group_by_entity` via `[TYPE:Name]` / first-40-chars fallback |
| Phase 2: contradiction resolution | Sort group by `confidence * (1 + recall_count)`; `contradiction_resolved = any(!is_latest)`; **does not DELETE or flip flags** |
| Phase 3: concept cards | Template `synthesize_summary` from up to 5 **latest** entries |
| Phase 4: export | `knowledge_core` table via `export_cards` |

Default `RipenConfig`: `dedup_threshold=0.95`, `min_entries_for_card=5`, `min_confidence=0.85`, `max_cards=50`, `export=true`.

**Implication for forget-lint:** clearing `is_latest=0` rows reduces ripen scan weight and contradiction noise **after** chains are retained in tombstone/plan artifacts (or after optional export). Do **not** invent a second contradiction scorer — dry-run class `contradiction_loser` should **mirror** `resolve_contradictions` sort key.

### 3.5 FTS / Qdrant residue

- `ensure_honeypot_fts_synced`: if any FTS row joins `is_latest=0`, full rebuild: `DELETE FROM honeypot_fts` then insert only `is_latest=1` ([vault.rs](../gzmo-core/src/memory/vault/)). So FTS dirtiness is transient; **SQLite honeypot table still holds 9 487 non-latest rows**.
- Qdrant: documented silent failure — upsert without supersede delete ([INFRASTRUCTURE_MAP.md](../docs/INFRASTRUCTURE_MAP.md)). forget-lint apply on lab vault should optionally emit qdrant-delete advice; living sync is out of scope for v1.

### 3.6 Proposed candidate classes (union, scored)

**Inputs:** lab vault path; policy; dry-run default.

| Class | Selection predicate (proposed) | Primary sources |
|-------|--------------------------------|-----------------|
| **A. Superseded** | `honeypot.is_latest = 0` | `supersede_honeypot`; CT101 9 487 rows |
| **B. Stale intermediate (latest)** | `is_latest=1` AND `recall_count=0` AND age(`promoted_at`) ∈ `[min_stale_days, max_stale_days]` AND `decay_class` ∈ policy allowlist (default Episodic, SessionDistill; **exclude** Structural / AbsoluteIdentity / Core) | spark archaeology window 14–90d for CuratedVault anchors; `DecayClass`; **policy must be conservative** — CuratedVault zero-recall is huge on CT101 |
| **C. Contradiction losers** | Within entity group (same `extract_entity_label` / `extract_primary_entity`), non-winners under ripen score among rows with any `!is_latest` sibling | `resolve_contradictions` |
| **D. Quarantine aged (opt-in)** | `quarantine_vault` older than N days | `list_quarantine`, store barrier |
| **E. Review-queue relation noise (opt-in)** | `honeypot_review_queue.reviewed=0` AND reason `relation_row` | CT101 queue schema |

**Advisory score (plan only):**

```text
forget_score =
    w_super   * I[is_latest=0]
  + w_recall  * 1/(1+recall_count)
  + w_age     * stale_sweetness(age_days, min, max)   // reuse spark.rs triangular helper concept
  + w_decay   * I[decay_class in aggressive set]
```

Hard blocks: `decay_class ∈ {Structural, AbsoluteIdentity, Core}` unless `--include-structural` (lab only). Refuse vault paths under `/opt/gzmo` or living appliance roots.

**Apply semantics (proposed):**

1. Default `plan` / `apply --dry-run`: emit candidates + tombstone records; **no DB mutation**.
2. `apply` (soft): for latest candidates → `is_latest=0` (+ optional `graph_rel`/`supersedes` bookkeeping) **or** move to quarantine; for already-superseded → mark forgotten in tombstone ledger / optional soft-delete table; sync FTS delete for rowid; **never** hard-DELETE without `--hard`.
3. Preserve chain navigability: prefer tombstone ledger over hard delete so `get_memory_chain` remains reconstructible from plan artifacts if rows are purged.

---

## 4. Proposed CLI + artifact / tombstone JSON schemas

Shape aligned with archaeology + honeypot-gate clap style + amp §1.4.

### 4.1 Commands

```bash
# Plan (always non-mutating)
forget-lint plan  --vault PATH/to/lab/vault.db \
  [--policy policy.toml] [--class superseded,stale,contradiction,quarantine] \
  [-o plan.json]

# Apply from frozen plan
forget-lint apply --vault PATH --plan plan.json \
  [--dry-run] [--hard] [--apply-quarantine] \
  [-o tombstones.jsonl]

# Optional inspect
forget-lint explain --vault PATH --id UUID   # chain via supersedes_id (read-only)
```

Refuse if `--vault` canonicalizes into living CT101 paths.

### 4.2 `plan.json` — `ltl.forget_lint.plan/v1`

```json
{
  "schema": "ltl.forget_lint.plan/v1",
  "created_at": "2026-07-20T00:00:00Z",
  "vault": "fixtures/tiny-vault.db",
  "dry_run": true,
  "policy": {
    "min_stale_days": 14,
    "max_stale_days": 90,
    "protect_decay_classes": ["Structural", "AbsoluteIdentity", "Core"],
    "allow_hard_delete": false
  },
  "census": {
    "honeypot_latest": 0,
    "honeypot_superseded": 0,
    "quarantine": 0
  },
  "candidates": [
    {
      "id": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
      "vault_id": "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
      "class": "superseded",
      "forget_score": 0.91,
      "decay_class": "CuratedVault",
      "origin": "ingest",
      "is_latest": 0,
      "recall_count": 0,
      "promoted_at": "2026-05-01T00:00:00Z",
      "supersedes_id": null,
      "successor_ids": ["ffffffff-….…"],
      "entity": "Firewall",
      "content_preview": "[AGENT:Firewall] …",
      "action": "tombstone",
      "reasons": ["is_latest=0", "ripen_loser_rank=3"]
    }
  ],
  "protected_skipped": [
    {"id": "…", "reason": "decay_class=Structural"}
  ],
  "blocks_living": true,
  "ok": true
}
```

`class` enum: `superseded | stale_intermediate | contradiction_loser | quarantine | review_queue`.  
`action` enum: `tombstone | quarantine | hard_delete | skip`.

### 4.3 `tombstones.jsonl` — one object per applied id

```json
{"schema":"ltl.forget_lint.tombstone/v1","id":"…","vault_id":"…","action":"tombstone","class":"superseded","at":"2026-07-20T12:00:00Z","plan_hash":"sha256:…","dry_run":false,"before":{"is_latest":0,"recall_count":0},"after":{"is_latest":0,"forgotten":true}}
```

Dry-run apply writes the same lines with `"dry_run": true` and no DB change (parity with honeypot-gate writing reports without promote).

### 4.4 Optional `policy.toml`

```toml
min_stale_days = 14
max_stale_days = 90
protect_decay_classes = ["Structural", "AbsoluteIdentity", "Core"]
classes = ["superseded", "contradiction_loser"]
w_super = 1.0
w_recall = 0.5
w_age = 0.3
refuse_paths = ["/opt/gzmo"]
```

---

## 5. Extract / port seams vs honeypot-gate

| Concern | honeypot-gate already does | forget-lint adds |
|---------|---------------------------|------------------|
| Ingress qualify (`confidence≥0.85`, source_file, boilerplate, relation ban) | `qualify::qualifies_for_honeypot` | Reuse optionally as “only forget rows that would still fail gate” hygiene — **do not fork** policy |
| Pairwise lifecycle classify | `lifecycle::classify_truth_pair` + `LifecycleAction` labels | Reuse classify to **explain** why a row was intermediate; gate never mutates vault |
| Vault audit | `audit::audit_vault` (read-only qualify report) | Plan/apply mutation + tombstone ledger |
| FTS / Qdrant cleanup | none | Soft apply + FTS row drop; optional qdrant delete advice |
| Contradiction productization | classify only | Align with ripen loser selection + **clear** residue |
| MCP `gzmo_memory_forget` | n/a | CLI is the little-tool shape; MCP can wrap later per MEMORY_ARCHITECTURE_SPEC §9 |

**Port recommendation:**

- **Do not** grow `honeypot-gate` into a mutator (CONTEXT: “Classify and gate … before they enter”; README: qualify + lifecycle classifier).
- **New piece** `forget-lint` (temp-bench / little-tools shape), depending on:
  - shared lifecycle classify (crate dep or thin copy already present in gate),
  - ripen scoring constants as a tiny shared lib or golden test against `resolve_contradictions`,
  - rusqlite read/write like `audit_vault` but with path guards.
- **In-tree** `gzmo-core` may later call the same lib from an MCP tool; living daemon cron remains out of v1.

```text
honeypot-gate:  facts.jsonl ──check──► gate-report.json
                 vault.db   ──audit──► gate-report.json (qualify only)

forget-lint:    vault.db ──plan──► plan.json
                vault.db + plan.json ──apply/--dry-run──► tombstones.jsonl
                         └──apply──► vault mutations (lab only)
```

---

## 6. Fixtures / smoke plan (lab vault only)

**Hard rule:** never CT101 import; never SSH mutate; fixture-smoke offline ([UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md) non-claim: feature-parity by import fakes uniqueness).

### 6.1 Fixture vault (`fixtures/tiny-vault.db`)

Minimal schema: `honeypot` (+ optional `quarantine_vault`, `honeypot_fts`) matching `SqliteVault::open` migration v3+ columns.

| Row id | Role | Expected plan class |
|--------|------|---------------------|
| H1 | latest CuratedVault, recall>0 | skip / low score |
| H2 | superseded (`is_latest=0`), H3.supersedes_id=H2 | `superseded` |
| H3 | contradict winner latest | skip |
| H4 | Extends sibling both latest, zero recall, old promoted_at | optional `stale_intermediate` if policy includes CuratedVault |
| H5 | Episodic latest, recall=0, age in window | `stale_intermediate` |
| H6 | Structural latest, recall=0, old | **protected_skipped** |
| H7–H9 | same entity cluster, mixed is_latest, low confidence×recall | `contradiction_loser` for losers |
| Q1 | quarantine aged | `quarantine` only with `--class quarantine` |

Seed via SQL in `fixtures/seed.sql` or a tiny Rust/Python builder — same pattern as honeypot-gate `fixtures/sample-facts.jsonl` but **SQLite**, because forget needs table semantics gate JSONL cannot express.

### 6.2 Smoke assertions

```bash
forget-lint plan --vault fixtures/tiny-vault.db -o /tmp/plan.json
# assert: H6 absent from candidates (or in protected_skipped)
# assert: H2 class=superseded
# assert: blocks_living=true, ok=true

forget-lint apply --vault fixtures/tiny-vault.db --plan /tmp/plan.json --dry-run -o /tmp/tombstones.jsonl
# assert: vault row counts unchanged; tombstones dry_run=true

forget-lint apply --vault fixtures/tiny-vault.db --plan /tmp/plan.json -o /tmp/tombstones.jsonl
# assert: H2 marked forgotten / removed per soft policy; H3 still is_latest=1
# assert: honeypot_fts has no is_latest=0 joins (if FTS present)
```

CI: fixture-smoke only. Optional golden: sort order of contradiction losers matches a copied unit test of ripen’s `confidence * (1+recall_count)`.

### 6.3 Lab soak (manual)

Copy **empty or tiny** `data-next/vault.db` — not CT101. Run plan → human review → apply. Never point at `/opt/gzmo/data/vault.db`.

---

## 7. Explicit non-goals

1. **Bulk import or mutate CT101 living vault** (`/opt/gzmo/data/vault.db`) — cite-only evidence mine.
2. **Replacing M5 ripen** or becoming the midnight `honeypot_ripen` job — ripen synthesizes; forget clears residue.
3. **Absorbing honeypot-gate** qualify/classify policy into a mutator binary.
4. **Auto-wiring into living-readiness GREEN math** or overnight daemon until an explicit lab recipe opts in.
5. **Hard-delete by default** or Qdrant/Neo4j live purge without separate guarded steps.
6. **Confusing spark stale-anchor reuse with forgetting** — spark needs stale CuratedVault; aggressive zero-recall purge of CuratedVault would poison serendipity.
7. **Implementing full MCP `gzmo_memory_forget` in v1** — CLI + schemas first; MCP is a later thin wrap.
8. **LLM adjudication** of forget candidates — stay rule-based like honeypot-gate.
9. **Chaos / PulseLoop / pedagogy coupling**.
10. **Closing Extends dual-latest as a bug** in v1 — Extends is intentional (`KeepBothAsLatest`); forget may optionally demote *zero-recall* extends under policy, not rewrite lifecycle semantics.

---

## 8. Ready-to-scaffold checklist

- [ ] Path guard refuses `/opt/gzmo` and living configs  
- [ ] `ltl.forget_lint.plan/v1` + tombstone schemas frozen  
- [ ] `fixtures/tiny-vault.db` + seed committed  
- [ ] Contradiction scoring parity test vs `ripen::resolve_contradictions`  
- [ ] Soft apply + FTS sync; `--hard` lab-only  
- [ ] Manifest stub in little-tools-lab (maturity: stub → partial)  
- [ ] Doc cross-link: gate = ingress; forget = egress hygiene; ripen = synthesize  

---

## Citation index

| Claim | Source |
|-------|--------|
| Product pick + shape | `research/ct101-vault-archaeology-2026-07-20.md` §forget-lint |
| Fact texts `690cb295`…`f8006497` | SSH CT101 `honeypot` SELECT 2026-07-20 |
| Layer counts | archaeology Snapshot + SSH `GROUP BY is_latest` |
| Supersede mutator | `lifecycle.rs` `supersede_honeypot` |
| Promote lifecycle wiring | `vault.rs` `promote_new_vault_truth` |
| Qualify ingress | `honeypot.rs` / gate `qualify.rs` |
| Decay enum | `types.rs` `DecayClass::half_life_days` |
| Decay runtime strings | `vault.rs` `half_life_from_decay_class` |
| Quarantine | `vault.rs` `store` / `list_quarantine` |
| Ripen phases + unused `dedup_threshold` | `ripen.rs` (doc vs `ripen_honeypot` body) |
| FTS drops non-latest | `vault.rs` `ensure_honeypot_fts_synced` |
| Spec forget tool | `MEMORY_ARCHITECTURE_SPEC.md` §9 `gzmo_memory_forget` |
| Qdrant stale points | `INFRASTRUCTURE_MAP.md` entanglement register |
| Gate non-goals / seams | `honeypot-gate/CONTEXT.md`, `main.rs`, `audit.rs` |
| Uniqueness boundary | `docs/UNIQUENESS_THESIS.md` |
