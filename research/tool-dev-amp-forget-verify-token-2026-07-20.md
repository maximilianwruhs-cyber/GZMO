# Tool-dev amp — forget-lint · verify-gates · token-economy

**Date:** 2026-07-20  
**Status:** Amplified build-spec research (no CLI scaffold in this pass)  
**Parent dig:** [ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md)  
**Boundary:** Cite CT101 evidence only. Operate on lab / `data-next` vault copies. Never bulk-import CT101 into workstation next ([UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md)).  
**Glossary:** piece / artifact / fixture / assembly — [little-tools-lab CONTEXT.md](../../little-tools-lab/CONTEXT.md).

**Dependency order for scaffolding:** `forget-lint` (memory hygiene) → `verify-gates` (quality) → `token-economy` (cost control).

---

## Subsystem atlas (shared)

```text
ingest / session_distill / dream / spark
        ↓ extract → prepare → verify (end-of-pipeline today)
  semantic_vault  ──confidence < 0.85──►  quarantine_vault
        ↓ promote (lifecycle: Duplicate|Extends|Contradicts|…)
     honeypot  (is_latest, supersedes_id, graph_rel, decay_class, recall_count)
        ├─ evidence (+ evidence_localize / evidence-locate piece)
        ↓ ripen M5 (group → resolve_contradictions → knowledge_core)
  knowledge_core  →  wiki emit (optional)
```

| Layer | Primary owner | Role for the three tools |
|-------|---------------|--------------------------|
| `semantic_vault` | `gzmo-core/src/memory/vault.rs` | Half-life + quarantine barrier (`confidence < 0.85`) |
| `honeypot` | `honeypot.rs`, `lifecycle.rs` | Supersession (`is_latest`), `graph_rel`, lifecycle kinds |
| `quarantine_vault` | `vault.rs` `list_quarantine` | Demote / HITL hold |
| `evidence` | `evidence*` + piece `evidence-locate` | Grounding for verify-gates |
| Ripen M5 | `memory/ripen.rs` `ripen_honeypot` | Contradiction scoring + concept-card export |
| Routing | `gzmo.toml` / `config/gzmo-next.toml` `[routing]` | Task→profile; token caps |
| Overnight jobs | CT101 `gzmo-daemon` | Living writer — **out of scope** for these pieces |

**CT101 snapshot (cite-only, SSH 2026-07-20):** vault 61 081 · honeypot latest 38 730 · honeypot all 48 217 · `is_latest=0` **9 487** · `supersedes_id` set **10 235** · quarantine 1 012 · evidence ~48k.

**Critical living-data note:** on CT101, **every** `is_latest=1` row currently has `recall_count=0` (38 730/38 730). Zero-recall alone cannot discriminate candidates for `forget-lint`.

**Tier-1 already extracted (do not re-ship):** `honeypot-gate`, `spark-link`, `rem-substrate`, `rrf-recall`, `synapse-tail`, `evidence-locate`, `faithfulness-judge`.

**Lab hot path today** (`little-tools-lab/scripts/cognition-smoke.sh`):

```text
session-distill → export → honeypot-gate → promote → spark-link → rrf-recall
  → evidence-locate batch(fixture) → meta
```

---

## 1. `forget-lint` — purposeful forgetting / intermediate-honeypot clearer

### 1.1 Problem (CT101)

| Fact id | Content (SSH read-only) | Source |
|---------|-------------------------|--------|
| `690cb295-a0a3-459c-ac21-e0a46d7c3553` | `[CONCEPT:Purposeful Forgetting]` systematic necessity | `the-cascading-honeypot-theorem-of-wisdom.md` |
| `26ad76d0-ff95-4b96-b287-d25e988335ec` | Purposeful Forgetting clears **intermediate honeypots** | same |
| `20811f83-db7e-4c4a-a3f0-cbbcb65baf3c` | `[CONCEPT:Lint/Maintenance]` as purposeful forgetting via contradiction resolve | `the-honeypot-compiler-architecture-distilling-dat.md` |
| `f8006497-e3d1-4329-9d93-6a24cd4b68f6` | Forgetting clears intermediate honeypots (systems thinking) | cascading theorem |

Wiki entities (CT101): `purposeful-forgetting.md`, `lint-maintenance.md`, `executable-wisdom.md`, `cascading-honeypot-theorem-of-wisdom.md`.

**Gap vs catalog:** `honeypot-gate` qualifies/classifies *before* promote ([CONTEXT.md](../../honeypot-gate/CONTEXT.md)); nothing ships **active forgetting** as a piece.

### 1.2 Primary-source inventory

| Path | Symbols / facts |
|------|-----------------|
| `gzmo-core/src/memory/ripen.rs` | `RipenConfig` defaults (`dedup_threshold=0.95`, `min_entries_for_card=5`, `min_confidence=0.85`, `max_cards=50`, `export`); `ripen_honeypot` → `group_by_entity` → `resolve_contradictions` → `export_cards` → `knowledge_core` |
| `gzmo-core/src/memory/lifecycle.rs` | `LifecycleKind::{Duplicate,Extends,Contradicts,Unrelated,Derives}`; `supersede_honeypot` sets `is_latest=0`; Extends keeps **both** `is_latest=1` |
| `gzmo-core/src/memory/honeypot.rs` | `HONEYPOT_MIN_CONFIDENCE=0.85`; `insert_honeypot_lifecycle`; `upsert_honeypot_row`; evidence upsert |
| `gzmo-core/src/types.rs` | `DecayClass::half_life_days` — Episodic 30, CuratedVault/SessionDistill 60, FlexibleIdentity 139, AbsoluteIdentity 693, Structural ∞ |
| `gzmo-core/src/memory/vault.rs` | `quarantine_vault` schema; promote quarantine barrier; `search_with_decay` filters `is_latest=1`; `list_quarantine`; `reinforce` bumps `recall_count` |
| `honeypot-gate/` | `qualify.rs`, `lifecycle.rs`, `audit.rs` — pre-promote only |
| CT101 schema | `honeypot.is_latest`, `supersedes_id`, `graph_rel`, `decay_class`, `recall_count`, `last_recalled_at` |

**Docs vs code (ripen):** module header advertises content-norm dedup + separate concept-card phase; implemented path groups by `[TYPE:Name]` label and synthesizes inside `resolve_contradictions` (`ripen.rs` L6–13 vs L83–107). Mirror **code**, not the outdated header.

**Winner formula (port this):** `confidence * (1.0 + recall_count)` — `ripen.rs` L165–172.

### 1.3 Algorithm / data model (candidate selection)

**Inputs:** lab SQLite vault path; policy; default dry-run.

**Candidate classes (union, scored):**

1. **Superseded (`is_latest=0`)** — already invisible to `search_with_decay`; still bloating FTS / ripen scans (`group_by_entity` reads all conf≥threshold). Prefer tombstone/archive after optional ripen export.
2. **Stale intermediate (`is_latest=1`)** — age(`promoted_at`) ∈ **14–90d** (reuse spark-link `stale_sweetness` window). Default decay filter: `Episodic` / `SessionDistill`; **protect** `Structural` / `AbsoluteIdentity`. CuratedVault needs stricter policy (CT101 has tens of thousands in-band).
3. **Extends siblings (true “intermediate honeypots”)** — `graph_rel='extends'` with both ends `is_latest=1` (lifecycle Extends action). After a ripen card exists, older/narrower sibling is forget fodder. This is the operational reading of facts `26ad76d0` / `f8006497`.
4. **Contradiction losers** — mirror ripen score; losers among entity clusters with any `is_latest=0`. Do **not** invent a second scorer.
5. **Quarantine demote (opt-in)** — `quarantine_vault` rows older than N days with no HITL; advice-only unless `--apply-quarantine`.

**Advisory score:**

`forget_score = w_age * stale_sweetness + w_recall * (1/(1+recall_count)) + w_super * (1 if is_latest=0 else 0) + w_extends`

Given CT101 `recall_count≡0` on all latest, weight **age + supersession + extends** heavily; keep `w_recall` for future reinforce traffic.

**Apply semantics:**

- Default plan: emit candidates; leave DB unchanged  
- Apply: soft `is_latest=0` and/or move to `quarantine_vault`; write `tombstones.jsonl`  
- Hard DELETE only with `--hard` + lab path guard  
- **Refuse** vault under `/opt/gzmo`

### 1.4 CLI + artifact schemas

```bash
forget-lint plan  --vault data-next/vault.db [--policy policy.toml] -o plan.json
forget-lint apply --vault data-next/vault.db --plan plan.json [--dry-run] [--hard]
```

**`plan.json`:**

```json
{
  "schema": "ltl.forget_lint.plan/v1",
  "vault": "…",
  "dry_run": true,
  "candidates": [
    {
      "id": "…",
      "class": "stale_intermediate|superseded|contradiction_loser|extends_intermediate|quarantine",
      "forget_score": 0.0,
      "decay_class": "Episodic",
      "is_latest": 1,
      "recall_count": 0,
      "supersedes_id": null,
      "graph_rel": null,
      "promoted_at": "…",
      "action": "tombstone|quarantine|skip"
    }
  ],
  "blocks_living": true,
  "ok": true
}
```

**`tombstones.jsonl`:** `{ "id", "action", "at", "plan_hash" }` per applied row.

### 1.5 Tier-1 / extract seams

| Piece / module | Reuse | Do not duplicate |
|----------------|-------|------------------|
| `honeypot-gate` | Optional filter: only forget rows that still qualify / failed audit | Copy `qualify.rs` / `lifecycle.rs` |
| `spark-link` | `stale_sweetness(days, 14, 90)` math | Spark LLM/hypothesize path; opposite product intent |
| Ripen formula | Winner sort key | Full overnight `ripen_honeypot` job |
| Soft-delete | `supersede_honeypot` semantics | Daemon promote loop |
| `rem-substrate` / `rrf-recall` | Out of path | — |

**Assembly:** optional hygiene stage after cognition-smoke honeypot-gate (stage 3), before long soak.

### 1.6 Fixtures / smoke

Fixture vault (≥10 honeypot rows): 3 superseded · 2 stale Episodic/SessionDistill in 14–90d · 1 Structural protected · 2 extends-pair both latest · 2 aged quarantine · 1 synthetic high-recall latest.

```bash
forget-lint plan --fixture fixtures/… -o /tmp/plan.json
# assert: Structural absent; superseded present; schema = ltl.forget_lint.plan/v1
forget-lint apply --vault fixtures/… --plan /tmp/plan.json --dry-run
```

CI: fixture-smoke only; **no SSH to CT101**. Unit: contradiction ranking parity vs `ripen::resolve_contradictions`.

### 1.7 Non-goals

- Mutate or bulk-import CT101 `/opt/gzmo/data/vault.db`  
- Replace ripen M5 or daemon overnight jobs  
- Auto-run on GREEN overnight gate  
- Hard-delete by default  
- Treat `recall_count=0` alone as sufficient on CT101-shaped data  
- DICE / PulseLoop coupling  

### 1.8 Ready-to-scaffold checklist

- [ ] New piece repo + `CONTEXT.md` (Cognition tier)  
- [ ] Lab path guard (`/opt/gzmo` refuse; prefer `data-next/`)  
- [ ] Freeze `ltl.forget_lint.plan/v1` + tombstone JSONL  
- [ ] Selectors: superseded, stale window, extends intermediates, ripen-parity losers, opt-in quarantine  
- [ ] Protect `Structural` / `AbsoluteIdentity`  
- [ ] Fixture vault + fixture-smoke  
- [ ] Contradiction scoring parity test vs `ripen.rs` L165–172  
- [ ] Manifest stub (justify 47th piece vs closed set of 46)  
- [ ] Optional cognition-smoke hook after honeypot-gate  
- [ ] Cite facts `690cb295`, `26ad76d0`, `20811f83`, `f8006497`  

---

## 2. `verify-gates` — mid-pipeline Analyze / Retrieve / Reason

### 2.1 Problem (CT101)

| Fact id | Content |
|---------|---------|
| `ffd73b42-8cf3-4806-a72f-22e55475234a` | `[PROJECT:Intermediate Verification Gates]` closes end-only verify gap |
| `11d7748d-6db1-4f0d-9db4-b1f1bb89442b` | No per-node Analyze / Retrieve / Reason gates |
| `663533a6-cf8f-45a1-ba57-63ae06835736` | Partial implementations exist but **disabled by default** |
| `b960e95c-c80d-4a7a-9089-d6ce5a6cd7fc` | Evidence-First RAG — each action cites `[E#]` from package |
| `acb8905d-e4cf-4f5a-a66f-a13ca422f78f` | Enforces citations; blocks unsafe path claims |
| `2102986b-e6a7-4c60-9c46-23280063bd8a` | `[CONCEPT:Tools Are Leaves]` (adjacent — separate `tool-chain` piece) |

All six: `source_file=drive-research-tinyfolder-gzmo-architecture-analysis-product.md`, CuratedVault, conf=1.0.

Wiki: `intermediate-verification-gates.md`, `verification-is-only-at-the-end-of-the-pipeline.md`, `evidence-first-rag.md`.

**Caveat:** fact `663533a6` / Pi chunk `#chunk26` mention `GZMO_ENABLE_GATES` stubs. Grep of this GZMO clone finds **no** those symbols. Today’s verify is end-of-pipeline `KgPromoter::verify` only (`kg_extract.rs` module doc L1; `ingest.rs` verify-on-merged). Dual gateway = extract vs verify **models**, not mid-stage Analyze/Retrieve/Reason (`KgPromoter::with_verify_gateway`).

**Gap vs catalog:** `etl-cli` / faithfulness-judge / evidence-locate cover extract→verify→promote and quote windows — **not** staged mid-pipeline node contracts. etl-cli is DEMOTE / superseded by session-distill (`PIECE_ELEVATION_MAP.md`).

### 2.2 Primary-source inventory

| Path | Symbols / facts |
|------|-----------------|
| `gzmo-core/src/memory/kg_extract.rs` | `KgGateConfig` (default `min_confidence=0.85`, `verify=true`), `KgPromoter`, `run_pipeline` / `run_merged_pipeline`, `verify()`, `verdict_passes`, `prepare_candidates` |
| `gzmo-core/src/memory/kg_promotion.rs` | `MIN_EVIDENCE_CHARS = 12` |
| `gzmo-core/src/ingest.rs` | `IngestEngine::new_with_verify`; localize after verify |
| `gzmo-core/src/memory/evidence_localize.rs` | ±1 sentence windows; LCS ≥12 → piece `evidence-locate` |
| `evidence-locate/` | `locate --body --quote`; `batch --fixture`; **No LLM**; `EvidenceSpan` JSON |
| `faithfulness-judge/` | Modes `context\|corpus\|both\|either`; dry-run word-overlap ≥0.6; `--live` optional LLM |
| `little-tools-lab/common/.../pipeline` | `FactStatus`, `attach_evidence` → Verified; promote requires Verified |
| `cognition-smoke.sh` | Stage 6 evidence-locate is **fixture quality floor**, not live Retrieve-gate |

**Pass criteria today** (`verdict_passes`): `supported` · `confidence >= min_confidence` (0.85) · if `require_evidence`: quote length ≥ 12.

### 2.3 Stage contracts

Shared packet / verdict:

```json
{
  "schema": "ltl.verify_gates.packet/v1",
  "stage": "analyze|retrieve|reason",
  "packet_id": "uuid",
  "source": { "kind": "document|session|recall_bundle|hypothesis", "uri": "…", "body_path": "…" },
  "evidence_package": [
    { "id": "E1", "text": "…", "char_start": 0, "char_end": 42, "origin": "source|retrieve|prior_gate" }
  ],
  "claims": [
    { "id": "C1", "text": "…", "cite": ["E1"], "kind": "observation|entity|relation|path|hypothesis" }
  ],
  "stage_payload": {}
}
```

```json
{
  "schema": "ltl.verify_gates.verdict/v1",
  "stage": "analyze",
  "packet_id": "uuid",
  "passed": true,
  "fail_closed": true,
  "claim_verdicts": [
    {
      "claim_id": "C1",
      "supported": true,
      "confidence": 0.9,
      "evidence_ids": ["E1"],
      "span": { "evidence_text": "…", "quote_verifier": "…", "char_start": 16, "char_end": 112 },
      "rejection_reasons": []
    }
  ],
  "stats": { "checked": 1, "passed": 1, "dropped": 0 }
}
```

**Global fail-closed:** every claim cites ≥1 existing `E#` (`b960e95c`) · unknown `E#` → fail (`acb8905d`) · path claims without localizable span when `body_path` set → fail · conf floor 0.85 · quote ≥12 when required.

```bash
verify-gates check --stage analyze|retrieve|reason --packet packet.json -o gate.json
verify-gates run   --stages analyze,retrieve,reason --packet packet.json -o report.json
```

| Stage | When | Pass | Does not |
|-------|------|------|----------|
| **analyze** | After candidate extract / before promote | Survive prep/noise spirit; cites localize via evidence-locate; optional faithfulness vs body | Call Qdrant/Neo4j; replace `KgPromoter::extract` |
| **retrieve** | After caller-provided recall bundle | Hits become stable `E#`; claims cite only those ids | Embed / RRF — compose **rrf-recall** upstream |
| **reason** | After hypothesis/answer draft | Claim entailed via faithfulness-judge (dry-run default); spans via evidence-locate; mirrors `verdict_passes` | Own the reasoner LLM |

### 2.4 Calling siblings without a second RAG stack

**Principle:** packet judge + subprocess orchestrator. Pieces do not library-import each other.

| Concern | Owner | verify-gates use |
|---------|-------|------------------|
| Quote → span | `evidence-locate locate` | Analyze + Reason |
| Claim entailed | `faithfulness-judge` | Reason (+ optional Analyze) |
| Hybrid recall | `rrf-recall` | **Upstream** of Retrieve |
| Production KG verify | `KgPromoter::verify` | Remains authoritative in core binaries |

**Anti-patterns:** embeddings inside verify-gates · reimplement `localize_evidence` · revive etl-cli as production ETL · silent auto-`vault_read` chaining (`2102986b` → separate `tool-chain`).

### 2.5 Assembly insertion

| Gate | Insert (GZMO-next / lab) |
|------|--------------------------|
| analyze | After distill export / beside honeypot-gate input |
| retrieve | After rrf-recall, before treating recall as trusted |
| reason | After spark hypothesis and/or before promote when grounding required |

Do **not** graft into CT101 daemon.

### 2.6 Tier-1 wiring

| Piece | Reuse | Do not duplicate |
|-------|-------|------------------|
| `evidence-locate` | CLI + `EvidenceSpan` JSON | In-process LCS copy |
| `faithfulness-judge` | modes + dry-run/live | New RAGAS stack |
| `rrf-recall` | Produce Retrieve hits | Second ranker |
| `honeypot-gate` | Qualify beside analyze | Absorb into honeypot-gate |
| `spark-link` | Hypotheses for Reason packets | Re-verify inside spark |
| `ltl-common` pipeline | `FactStatus`, `attach_evidence` | New status enum |

### 2.7 Fixtures / smoke

Six packets: analyze pass/fail-missing-cite · retrieve pass/fail-foreign-cite · reason pass/fail-unsupported. Reuse bodies from `evidence-locate/fixtures` and `faithfulness-judge/fixtures`.

Optional cognition-smoke Stage 5b/6b behind binary presence; keep stage 6 evidence-locate batch as leaf green check.

### 2.8 Non-goals

- Second RAG stack  
- CT101 daemon graft / vault import  
- Replace `KgPromoter::verify` on day one  
- Revive etl-cli as production ETL  
- Implement Tool Chaining (`2102986b`)  
- Treat fixture evidence-locate batch as Retrieve-gate coverage  
- Expand closed 46 without catalog justification  

### 2.9 Ready-to-scaffold checklist

- [ ] Freeze `ltl.verify_gates.packet/v1` + `verdict/v1`  
- [ ] CLI `check` / `run` / `batch`; fail-closed exit codes  
- [ ] Subprocess only: evidence-locate + faithfulness-judge  
- [ ] Six fixture packets with expected pass/fail  
- [ ] Doc: relationship to `KgPromoter::verify` / dual-gateway (not a replace)  
- [ ] Manifest stub: Quality tier; related evidence-locate, faithfulness-judge, rrf-recall  
- [ ] Optional cognition-smoke hook  
- [ ] Catalog exception for 47th piece  
- [ ] Cite CT101 fact ids; no vault copy  

---

## 3. `token-economy` — reactive token budget / Co-Saving router

### 3.1 Problem (CT101)

| Fact id | Content |
|---------|---------|
| `06c23921-0378-4630-812c-f0a1432ee5e0` | Reactive Token Economy: budget, route, compress, cache context dynamically |
| `b3be19fc-2e44-431a-b43c-7cb08eba12b4` | Internalizes economic constraints into reasoning + infrastructure |
| `5d774056-a762-4def-82db-a76c59659845` | Co-Saving ~**50.85%** token cut vs baseline MAS (literature extract) |
| `9c827e76-02f2-4e3b-a05b-45079d250475` | TALE = Token-Budget-Aware LLM Reasoning |
| `bfd75315-d318-49d5-b75f-e148545c2233` | TALE zero-shot budget estimator |
| `2cdff379-5c3a-4835-844e-49dfd85e7a8b` | TALE adjusts reasoning tokens by problem complexity |

Wiki: `reactive-token-economy.md`, `co-saving.md`, `tale.md`, `tiered-model-routing.md`, `semantic-caching.md`, `prefix-caching-optimization.md`, `three-layer-budget-enforcement.md`.  
Sources: `drive-research-agentic-token-economy-blueprint-micro0{1,2,3}.md`.

**Do not claim** Co-Saving 50.85% or TALE ~68% CoT cuts as measured on GZMO without a harness — they are literature facts in the vault.

### 3.2 Primary-source inventory — routing + token caps

| Config | Keys |
|--------|------|
| `GZMO/gzmo.toml` | `[spark] max_tokens_hypothesis/verify=4096`; `[engine.local] max_tokens=24576`; `[context_memory] scratch_max_tokens=2000`, `context_length=131072`; `[routing.mappings]` dream/spark→local, distill_extract→librarian, ingest_extract→local_deterministic |
| `config/gzmo-next.toml` | Same spark/context caps; **all** cognition mappings → `"local"` (no librarian split) |
| `config/gzmo-next-fused.toml` | Generated by config-fuse; `[engine] max_tokens=820` (Lorenz); `[routing.rapl] fits_budget=false` |
| `gzmo-core/src/config.rs` | `TaskKind` enum; `RoutingConfig` (“Obolus, the Economy Organ” = **static routing alias**, not IPW product); `ContextMemoryConfig::hot_budget_tokens` |
| Call sites | `gateway.rs` `effective_max_tokens`; `spark.rs` phase caps; `context.rs` chars/token ≈3.5; `scratch.rs` inject cap; dreams/distill use routed profile caps |

### 3.3 Estimator inputs (grounded)

| Input | Grounded keys | Use |
|-------|---------------|-----|
| Task class | `TaskKind` / mapping keys | Baseline profile + phase cap |
| Message size | `estimate_text_tokens` ≈ chars/3.5 (`context.rs`) | Scale output / scratch pressure |
| Profile | local / cloud / librarian / local_deterministic | Cap ≤ profile `max_tokens` |
| Spark phase | `[spark].max_tokens_*` | Structured caps 4096 |
| Context pack | `[context_memory].*` | Hot budget + inject |
| Lorenz / RAPL | fused `max_tokens`, `[routing.rapl].fits_budget` | Soft prior only |

**Default advise table:**

| task_class | advise `max_tokens` | routing_hint (next) |
|------------|--------------------:|---------------------|
| spark_hypothesis / verify | 4096 | local |
| dream_* | ≤ 24576 | local |
| distill_extract / summary | ≤ 4096 (librarian floor) | local (next) / librarian (root) |
| distill_verify | ≤ local floor | local |
| chat | practical lower band ≤2048–8192 | local |
| fused lorenz prior | 820 (soft) | local |

### 3.4 CLI + artifact schemas

```bash
token-economy estimate --task distill_verify --chars 12000 --profile local -o budget.json
token-economy snippet  --budget budget.json -o routing-hint.toml
token-economy estimate --fixture fixtures/small-chat.json
```

**`budget.json`:**

```json
{
  "schema": "ltl.token_economy.budget/v1",
  "tool": "token-economy",
  "task_class": "distill_verify",
  "profile": "local",
  "inputs": {
    "message_chars": 12000,
    "estimate_input_tokens": 3433,
    "chars_per_token": 3.5
  },
  "recommend": {
    "max_tokens": 4096,
    "scratch_max_tokens": 2000,
    "context_length": 131072,
    "archive_threshold": 0.90,
    "response_reserve": 0.10,
    "routing_hint": "local",
    "compress": ["archive_hot_over_archive_threshold", "scratch_inject_cap", "prefer_pointers_over_raw_dumps"],
    "cache": ["stable_system_prefix_for_prefix_cache", "semantic_cache_check_before_llm", "reuse_evidence_package"],
    "co_saving": ["skip_redundant_mas_hops_when_shortcut_safe"]
  },
  "citations": {
    "rte": "06c23921-0378-4630-812c-f0a1432ee5e0",
    "tale": "9c827e76-02f2-4e3b-a05b-45079d250475",
    "co_saving": "5d774056-a762-4def-82db-a76c59659845"
  },
  "co_saving_note": "advisory literature % only — do not claim measured 50.85% without harness",
  "blocks_distill": false,
  "ok": true
}
```

Advisory TOML (human merge only — mirror fuse/lorenz; never auto-write live `gzmo.toml`):

```toml
# emitted by token-economy — human merge only
[routing.profiles.lab_token_tight]
provider = "local"
max_tokens = 2048
temperature = 0.2
```

### 3.5 Co-Saving / TALE as advisory only

Map CT101 themes → hints in `budget.json`, not a cloud MAS router:

| Theme | Emit |
|-------|------|
| Graph shortcuts / fewer hops | `co_saving` hints |
| Tiered routing | light profile for extract/summary, heavy for verify |
| Context budget pattern | align `archive_threshold` / pointer-not-dump |
| Prefix / semantic cache | cache hints |
| TALE prompt budget | optional `prompt_budget` for CoT-heavy tasks |

**Do not** graft CT101 cloud-first background routing into lab defaults (next maps everything local).

### 3.6 Contrast with Obolus (IPW)

| | **Obolus** | **`token-economy`** |
|--|------------|---------------------|
| Question | Best answers **per joule / €** | Best quality under **token/context** budget |
| Formula | `z = quality / (joules × price_factor)` ([Obolus/README.md](../../Obolus/README.md)) | TALE-style estimate + static GZMO caps |
| Energy | RAPL / labeled estimate | Out of scope (may soft-read fused RAPL) |
| Artifact | bench JSON + recommend | `budget.json` + TOML snippet |

Name collision: `RoutingConfig` comment “Obolus, the Economy Organ” = static task→engine table, **not** the IPW product.

### 3.7 Tier-1 / assembly wiring

| Piece / recipe | Role |
|----------------|------|
| `lorenz-map` / `config-fuse` | Emit/merge `[engine] max_tokens` + optional RAPL into `*-fused.toml` |
| `cognition-smoke` | No token stage today; optional `--token-budget` before LLM-heavy extract/verify |
| `spark-link` / `session-distill` | Consumers of caps via engine, not re-implementers |
| Obolus / Arena | Soft input only; `blocks_distill` must stay **false** |

### 3.8 Fixtures / smoke

- `small-chat.json` → tight max_tokens  
- `heavy-dream.json` → allow up to engine floor / warn on fused 820  
- `spark-verify.json` → expect 4096  
- `distill-librarian.json` → expect ≤4096 when profile=librarian  
- Assert `blocks_distill=false`, schema id, divergence on `--chars`  
- Snippet parses as TOML; never writes outside `-o`  

### 3.9 Non-goals

- Graft CT101 routing / cloud verify wholesale  
- Import CT101 vault into `data-next`  
- Auto-edit live `gzmo.toml`  
- Re-ship Obolus RAPL / € / IpW  
- Claim literature % as GZMO measurements without harness  
- Second agent loop / LangGraph MAS  
- Replace `context.rs` / gateway — piece emits policy only  

### 3.10 Ready-to-scaffold checklist

- [ ] New piece `token-economy/` with CLI `estimate` / `snippet`  
- [ ] Freeze `ltl.token_economy.budget/v1`  
- [ ] Estimator table keyed by `TaskKind` + spark/context keys  
- [ ] Fixture matrix + divergence asserts  
- [ ] TOML snippet “human merge only”; refuse default live paths  
- [ ] Manifest stub (Ops or Cognition; maturity stub→partial)  
- [ ] Optional cognition-smoke `--token-budget`  
- [ ] Docs: contrast Obolus IPW; clarify RoutingConfig “Obolus” alias  
- [ ] Cite CT101 ids; no vault import  
- [ ] Soft-read fused RAPL / Lorenz without requiring RAPL  
- [ ] Keep `blocks_distill=false` invariant  

---

## Assembly sketch (future)

```text
session-distill
  → [verify-gates analyze]          # optional
  → honeypot-gate
  → [forget-lint plan|apply]        # optional hygiene on lab vault
  → promote
  → [token-economy estimate]        # optional advisory before LLM-heavy
  → spark-link
  → rrf-recall
  → [verify-gates retrieve|reason]  # optional
  → evidence-locate batch
  → meta
```

Vocabulary and closed-set rules: [little-tools-lab CONTEXT.md](../../little-tools-lab/CONTEXT.md). Each new piece is an exceptional 47th unless the catalog absorbs scope — justify in a catalog PR when scaffolding.

---

## Provenance

| Kind | Source |
|------|--------|
| Parallel research briefs | [forget-lint-primary-sources-2026-07-20.md](./forget-lint-primary-sources-2026-07-20.md) · [verify-gates-research-brief-2026-07-20.md](./verify-gates-research-brief-2026-07-20.md) · [token-economy-primary-sources-2026-07-20.md](./token-economy-primary-sources-2026-07-20.md) |
| CT101 SQL | `/opt/gzmo/data/vault.db` honeypot facts + counts (read-only SSH) |
| CT101 wiki | `/opt/gzmo/wiki/entities/*`, `/opt/gzmo/wiki/sources/*` |
| Code | `gzmo-core` memory/{ripen,honeypot,lifecycle,vault,kg_extract,evidence_localize}, config/gateway/context/spark; pieces honeypot-gate, evidence-locate, faithfulness-judge; Obolus README; config-fuse / lorenz-map |
| Lab | `cognition-smoke.sh`, CONTEXT.md, ASSEMBLIES.md |
| Prior dig | [ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md) |
| Thesis | [UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md) |
