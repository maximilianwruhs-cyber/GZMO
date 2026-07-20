# Tool-dev amp — forget-lint · verify-gates · token-economy

**Date:** 2026-07-20  
**Status:** Build-spec research (no CLI scaffold in this pass)  
**Parent dig:** [ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md)  
**Boundary:** Cite CT101 evidence only. Operate on lab / `data-next` vault copies. Never bulk-import CT101 into workstation next ([UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md), ADR-0003).  
**Glossary:** piece / artifact / fixture / assembly — [little-tools-lab CONTEXT.md](../../little-tools-lab/CONTEXT.md).

---

## Subsystem atlas (shared)

```text
ingest / session_distill / dream
        ↓
  semantic_vault  ──confidence < 0.85──►  quarantine_vault
        ↓ promote
     honeypot  (is_latest, supersedes_id, decay_class, recall_count)
        ↓ ripen M5
  knowledge_core  →  wiki emit (optional)
        ↑
  evidence (+ evidence_localize / evidence-locate piece)
```

| Layer | Primary owner | Role for the three tools |
|-------|---------------|--------------------------|
| `semantic_vault` | `gzmo-core/src/memory/vault.rs` | Half-life + quarantine barrier (`confidence < 0.85`) |
| `honeypot` | `honeypot.rs`, `lifecycle.rs` | Supersession (`is_latest`), lifecycle kinds |
| `quarantine_vault` | `vault.rs` `list_quarantine` | Demote / HITL hold |
| `evidence` | `evidence*` + piece `evidence-locate` | Grounding for verify-gates |
| Ripen M5 | `memory/ripen.rs` `ripen_honeypot` | Dedup → contradiction → concept cards → export |
| Routing | `config/gzmo-next.toml` `[routing]` | Profile → engine; token caps nearby |
| Overnight jobs | CT101 `gzmo-daemon` | Living writer — **out of scope** for these pieces |

**CT101 snapshot (cite-only):** ~61k vault / ~39k latest honeypot / ~48k evidence / ~1k quarantine — archaeology §Snapshot.

**Tier-1 already extracted (do not re-ship):** `honeypot-gate`, `spark-link`, `rem-substrate`, `rrf-recall`, `synapse-tail`, `evidence-locate`, `faithfulness-judge`.

**Dependency order for scaffolding:** `forget-lint` (hygiene) → `verify-gates` (quality) → `token-economy` (cost).

---

## 1. `forget-lint` — purposeful forgetting / intermediate-honeypot clearer

### 1.1 Problem (CT101)

| Fact id | Content (SSH read-only, 2026-07-20) |
|---------|--------------------------------------|
| `690cb295` | `[CONCEPT:Purposeful Forgetting]` systematic necessity |
| `26ad76d0` | Purposeful Forgetting clears **intermediate honeypots** |
| `20811f83` | `[CONCEPT:Lint/Maintenance]` as purposeful forgetting / contradiction resolve |
| `f8006497` | Forgetting clears intermediate honeypots (systems-thinking) |

Sources named in archaeology: cascading-honeypot / honeypot-compiler research ingest.  
**Gap vs catalog:** `honeypot-gate` qualifies/classifies before promote; nothing ships **forgetting as product**.

### 1.2 Primary-source inventory

| Path | Symbols / facts |
|------|-----------------|
| `gzmo-core/src/memory/ripen.rs` | `RipenConfig` (`dedup_threshold=0.95`, `min_entries_for_card=5`, `min_confidence=0.85`, `max_cards=50`, `export`); phases: group → `resolve_contradictions` (uses `is_latest=0`) → concept cards → `knowledge_core` export |
| `gzmo-core/src/memory/lifecycle.rs` | `LifecycleKind::{Duplicate,Extends,Contradicts,Unrelated,Derives}`; contradict sets prior `is_latest = 0` |
| `gzmo-core/src/memory/honeypot.rs` | Insert/promote paths; `is_latest`, `recall_count`, `supersedes_id` columns |
| `gzmo-core/src/types.rs` | `DecayClass` + `half_life_days()` (Episodic 30 … Structural ∞) |
| `gzmo-core/src/memory/vault.rs` | `quarantine_vault` schema; `list_quarantine`; confidence quarantine inserts |
| `gzmo-core/src/memory/vault_backend.rs` | Decay formula comment; `<0.85` quarantine barrier |
| CT101 schema | `honeypot.is_latest`, `supersedes_id`, `decay_class`, `recall_count`, `last_recalled_at`, `verify_pass` |
| `honeypot-gate/CONTEXT.md` | Stage-2 of cognition-smoke: classify → `gate-report.json`; **not** active forget |

### 1.3 Algorithm / data model (candidate selection)

**Inputs:** path to lab SQLite vault; policy flags; `--dry-run` default.

**Candidate classes (union, scored):**

1. **Superseded** — `honeypot.is_latest = 0` (already tombstoned for recall; may still bloat FTS / ripen scans). Prefer archive/delete-after-export over re-flipping.
2. **Stale intermediate** — `is_latest = 1` AND `recall_count = 0` AND age(`promoted_at`) ∈ configurable sweet band (archaeology spark used 14–90d for CuratedVault) AND `decay_class` ∈ {Episodic, SessionDistill} by default (protect Structural / AbsoluteIdentity).
3. **Contradiction losers** — same entity cluster as ripen phase 2 would drop (low `confidence × recall_count` among `is_latest=0` siblings) — dry-run should **mirror** `resolve_contradictions` scoring, not invent a second scorer.
4. **Quarantine demote (opt-in)** — `quarantine_vault` rows older than N days with no HITL promote; emit advice only unless `--apply-quarantine`.

**Scoring (advisory):** `forget_score = w_age * stale_sweetness + w_recall * (1/(1+recall_count)) + w_super * (1 if is_latest=0 else 0)`. Do not auto-touch `Structural`.

**Apply semantics:**

- Default: write `tombstones.jsonl` + leave rows (plan mode).
- `--apply`: set `is_latest=0` if still 1, or move to `quarantine_vault` / soft-delete table — **never** hard-DELETE without `--hard` and lab-only path guard.
- Refuse if vault path resolves under `/opt/gzmo` or matches living appliance config.

### 1.4 CLI + artifact schemas

```bash
forget-lint plan  --vault data-next/vault.db [--policy policy.toml] -o plan.json
forget-lint apply --vault data-next/vault.db --plan plan.json [--dry-run] [--hard]
```

**`plan.json` (artifact):**

```json
{
  "schema": "ltl.forget_lint.plan/v1",
  "vault": "…",
  "dry_run": true,
  "candidates": [
    {
      "id": "…",
      "class": "stale_intermediate|superseded|contradiction_loser|quarantine",
      "forget_score": 0.0,
      "decay_class": "Episodic",
      "is_latest": 1,
      "recall_count": 0,
      "action": "tombstone|quarantine|skip"
    }
  ],
  "blocks_living": true,
  "ok": true
}
```

**`tombstones.jsonl`:** one JSON object per applied id `{id, action, at, plan_hash}`.

### 1.5 Tier-1 / extract seams

| Piece | Call / reuse | Do not duplicate |
|-------|--------------|------------------|
| `honeypot-gate` | Optional: only forget facts that previously **passed** gate (promote hygiene) | Gate classify policy |
| `ripen` (in-tree) | Share contradiction winner formula | Full M5 export job |
| `spark-link` / `rrf-recall` | Out of path | — |

**Port shape:** new little-tools **piece** repo (Rust or Python), fixture vault under `fixtures/tiny-vault.db`; assembly hook after cognition-smoke gate stage (hygiene before next distill soak).

### 1.6 Fixtures / smoke

- Fixture: 10 honeypot rows (3 superseded, 2 stale zero-recall, 1 Structural protected, 2 quarantine).
- `forget-lint plan --fixture …` → expect Structural absent from apply list; superseded listed.
- CI: fixture-smoke only; no SSH to CT101.

### 1.7 Non-goals

- Bulk import / mutate CT101 living vault  
- Replacing ripen or daemon jobs  
- Auto-running on GREEN overnight gate  
- Inventing `DICE_MASTER_*` / chaos PulseLoop coupling  

### 1.8 Ready-to-scaffold checklist

- [ ] Lab path guard implemented  
- [ ] Plan schema frozen  
- [ ] Fixture vault committed  
- [ ] Contradiction scoring parity test vs `ripen::resolve_contradictions`  
- [ ] Manifest stub in little-tools-lab (maturity: stub → partial)

---

## 2. `verify-gates` — mid-pipeline Analyze / Retrieve / Reason

### 2.1 Problem (CT101)

| Fact id | Content |
|---------|---------|
| `ffd73b42` | `[PROJECT:Intermediate Verification Gates]` — closes end-only verify gap |
| `11d7748d` | No per-node Analyze / Retrieve / Reason gates |
| `663533a6` | Partial implementations exist but **disabled by default** |
| `b960e95c` | Evidence-First RAG — cite `[E#]` from evidence package |
| `acb8905d` | Enforces citations; blocks unsafe path claims |
| `2102986b` | `[CONCEPT:Tools Are Leaves]` architectural gap (adjacent) |

**Gap vs catalog:** `etl-cli` / in-tree `KgPipeline::verify` / `faithfulness-judge` / `evidence-locate` cover extract→verify→promote and quote windows — **not** staged mid-pipeline node contracts.

### 2.2 Primary-source inventory

| Path | Symbols / facts |
|------|-----------------|
| `gzmo-core/src/memory/kg_extract.rs` | `KgPipeline`, `gate.verify`, `verify()`, `merge_extractions_pre_verify`, extract→prepare→verify→promote |
| `gzmo-core/src/ingest.rs` | End-of-ingest verify wiring (pipeline consumer) |
| `gzmo-core/src/memory/evidence_localize.rs` (ported) | Quote window logic → piece `evidence-locate` |
| `evidence-locate/` | CLI `batch --fixture`; no LLM; LCS fallback |
| `faithfulness-judge/` | Probes YAML; modes; Quality tier wave 1 |
| `config/gzmo-next.toml` | `max_tokens_verify = 4096`; `*_verify` routing maps → `local` |

Today verify is **one** LLM (or deterministic) gate near the end of extract, not three named stages.

### 2.3 Stage contracts

Shared **packet** (artifact in / artifact out):

```json
{
  "schema": "ltl.verify_gates.packet/v1",
  "stage": "analyze|retrieve|reason",
  "claim": "…",
  "context": {"source_text": "…", "evidence": [{"id": "E1", "quote": "…"}]},
  "citations": ["E1"],
  "meta": {"task_id": "…", "profile": "ingest_verify"}
}
```

| Stage | Pass criteria | Fail / HOLD |
|-------|---------------|-------------|
| **analyze** | Claim is well-formed; entity anchors parseable; no empty claim | Malformed packet; Derives-without-verify (`LifecycleKind::Derives` spirit) |
| **retrieve** | ≥1 evidence hit via `evidence-locate` (or pre-supplied `[E#]` with localized quote) | Zero evidence; quote not in source |
| **reason** | `faithfulness-judge` (or equivalent probe) PASS on claim∪evidence; citations required (`b960e95c`) | Hallucinated path; missing citations |

**CLI sketch:**

```bash
verify-gates check --stage analyze|retrieve|reason --packet packet.json -o gate.json
verify-gates run   --stages analyze,retrieve,reason --packet packet.json -o report.json
```

`gate.json`: `{schema, stage, verdict: PASS|FAIL|HOLD, reasons[], evidence_refs[], blocks_promote: bool}`.

### 2.4 Tier-1 wiring (no second RAG)

```text
packet ──analyze (rules)──► packet'
        ──retrieve──► evidence-locate batch (fixture or source_text)
        ──reason──► faithfulness-judge --mode … 
```

- **Retrieve** calls `evidence-locate` as a leaf (Tools Are Leaves — `2102986b`): follow refs by running the piece, do not embed a vector RAG inside verify-gates.  
- **Reason** calls `faithfulness-judge` leaf; do not reimplement probe math.  
- In-tree `KgPipeline::verify` remains the **production** LLM verify; verify-gates is the **assembly-visible** staged contract for GZMO-next recipes and lab cognition-smoke.

### 2.5 Assembly insertion

Adjacent to `cognition-smoke` / etl:

1. session-distill export  
2. `honeypot-gate check`  
3. **`verify-gates run`** (new) on candidate facts before promote  
4. spark-link / rrf-recall smoke  

Living CT101 daemon path stays unchanged until an explicit promote recipe opts in.

### 2.6 Fixtures / smoke

- Packets: one PASS triad, one FAIL retrieve (no quote), one FAIL reason (uncited claim).  
- Fixture-smoke offline; optional `--live` against local Prime verify profile.

### 2.7 Non-goals

- Replacing `KgPipeline` overnight on CT101  
- Building a new embed/RAG stack  
- Wiring into `living-readiness-gate` GREEN math  
- Auto-blocking distill from Arena/IpW/Forge advice (separate boundary)

### 2.8 Ready-to-scaffold checklist

- [ ] Packet schema frozen  
- [ ] Leaf CLIs invoked via subprocess + JSON paths only  
- [ ] Fixture packets + expected verdicts  
- [ ] Doc: relationship to `kg_extract::KgPipeline::verify`  
- [ ] Manifest stub (Quality tier)

---

## 3. `token-economy` — reactive budget / Co-Saving router

### 3.1 Problem (CT101)

| Fact id | Content |
|---------|---------|
| `06c23921` | Reactive Token Economy — budget, route, compress, cache dynamically |
| `b3be19fc` | Internalize economic constraints into agent loop + infra |
| `c605279c` | Approach Pareto frontier of cost × quality |
| `5d774056` | Co-Saving ~**50.85%** multi-agent token reduction vs baseline MAS |
| `9c827e76` | **TALE** — Token-Budget-Aware LLM Reasoning |

**Gap vs catalog:** Obolus / IpW answers **smart per watt**; this answers **smart per token/context**. Do not re-ship Obolus.

### 3.2 Primary-source inventory

| Path | Symbols / facts |
|------|-----------------|
| `config/gzmo-next.toml` | `max_tokens_hypothesis/verify = 4096`; `scratch_max_tokens = 2000`; `context_length = 131072`; `[routing]` + `[routing.mappings]` (`dream_*`, `spark_*`, `ingest_*`, `distill_*` → `local`) |
| `gzmo-core/src/context.rs` | `max_tokens`, archive at ~90% budget, trim |
| `gzmo-core/src/gateway.rs` | `GatewayConfig.max_tokens`, chaos overrides |
| Dreams / spark | Use routing maps + verify token caps (config-driven) |
| IpW / Arena | `scripts/ipw-route-*.sh`, `docs/OBOLUS_*` — watt/€ advice; `blocks_distill=false` |

### 3.3 Contrast: Obolus vs token-economy

| | Obolus / IpW | token-economy |
|--|--------------|---------------|
| Unit | joules / € / route class | tokens / context pack size |
| Artifact | `ipw-router/latest.json`, Arena euro-night | `budget.json` + optional `[routing]` snippet |
| Action | advise engine class (chat vs heavy_bench) | advise max_tokens, scratch trim, cache/compress hints |
| Living | outside daemon by default | same — advisory lab piece |

### 3.4 Estimator + CLI

**Inputs:** `task_class` ∈ {chat, distill, dream, spark, ingest, heavy_bench}; `message_chars`; `profile` (routing map key); optional prior `budget.json`.

**Outputs (advisory):**

```json
{
  "schema": "ltl.token_economy.budget/v1",
  "task_class": "distill",
  "estimate_tokens": 0,
  "recommend": {
    "max_tokens": 4096,
    "scratch_max_tokens": 2000,
    "compress": ["drop_system_repeats", "archive_hot_over_90pct"],
    "cache": ["reuse_evidence_package"],
    "routing_hint": "local"
  },
  "co_saving_note": "advisory — cite 5d774056; do not claim measured 50.85% without fixture proof",
  "blocks_distill": false,
  "ok": true
}
```

Optional TOML snippet (never auto-write live `gzmo.toml`):

```toml
# emitted by token-economy — human merge only
[routing.profiles.lab_tight]
max_tokens = 2048
```

```bash
token-economy estimate --task distill --chars 12000 --profile distill_verify -o budget.json
token-economy snippet --budget budget.json -o routing-hint.toml
```

### 3.5 Tier-1 / assembly

- May **read** IpW advice as soft input (watt class → tighten tokens) but must not flip `blocks_distill`.  
- Context trim ideas align with `context.rs` archive trigger — piece emits policy, GZMO applies later.  
- Assembly: optional stage in cognition-smoke **before** LLM-heavy extract/verify to print budget advice into the meta report.

### 3.6 Fixtures / smoke

- Fixture matrix: small chat vs heavy distill char counts → diverging `max_tokens` recommendations.  
- Assert `blocks_distill=false` always in artifact.  
- No cloud routing table copy from CT101.

### 3.7 Non-goals

- Grafting CT101 cloud verify routes wholesale  
- Auto-editing `~/.gzmo/gzmo.toml` or living `/opt/gzmo` config  
- Replacing Obolus RAPL / € night  
- Claiming Co-Saving % without measured fixture harness  

### 3.8 Ready-to-scaffold checklist

- [ ] Estimator table documented (task_class → defaults from gzmo-next.toml)  
- [ ] budget schema frozen  
- [ ] Fixture divergence test (chat vs heavy)  
- [ ] Explicit “human merge only” on TOML snippets  
- [ ] Manifest stub (Ops or Cognition tier)

---

## Assembly sketch (future little-tools-lab)

```text
cognition-smoke (adjacent)
  session-distill → honeypot-gate → verify-gates → (optional) token-economy estimate
                                                 → spark-link / rrf-recall
nightly lab hygiene (separate recipe)
  forget-lint plan|apply --vault data-next/…   # never CT101
```

Vocabulary: each new repo is a **piece**; JSON outputs are **artifacts**; committed DBs/packets are **fixtures**; bash chain is an **assembly**.

---

## Cross-tool notes

1. **forget-lint** reduces honeypot entropy so ripen/verify see less junk — run before long lab soaks.  
2. **verify-gates** raises promote quality; pairs with Evidence-First facts (`b960e95c`).  
3. **token-economy** caps spend on verify/extract once gates exist (verify stages multiply LLM calls if naïvely enabled).  
4. All three stay **off** `living-readiness-gate` / product A stranger install until operators promote recipes.

---

## Out of scope (this research)

- Scaffolding the three CLIs  
- Remaining archaeology candidates (tool-chain, PDU, phantom-drive, …)  
- Importing CT101 vault into `data-next`  
- Editing little-tools catalog / archaeology files beyond citation  

---

## Citation index

| Claim area | Sources |
|------------|---------|
| Executive picks | `research/ct101-vault-archaeology-2026-07-20.md` §Executive picks |
| CT101 fact texts | SSH `sqlite3 /opt/gzmo/data/vault.db` honeypot rows (2026-07-20) |
| Ripen M5 | `gzmo-core/src/memory/ripen.rs` |
| Lifecycle / supersession | `gzmo-core/src/memory/lifecycle.rs` |
| Decay | `gzmo-core/src/types.rs` `DecayClass::half_life_days` |
| Quarantine | `gzmo-core/src/memory/vault.rs` |
| End verify | `gzmo-core/src/memory/kg_extract.rs` `KgPipeline` |
| Routing / tokens | `config/gzmo-next.toml`, `gzmo-core/src/context.rs`, `gateway.rs` |
| Tier-1 pieces | `honeypot-gate/CONTEXT.md`, `evidence-locate/`, `faithfulness-judge/` |
| Lab language | `little-tools-lab/CONTEXT.md` |
