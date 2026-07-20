# Tool-dev amp — forget-lint · verify-gates · token-economy

**Date:** 2026-07-20  
**Status:** Synthesis index + scaffold order (no CLI in this pass)  
**Parent dig:** [ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md)  
**Boundary:** Cite CT101 only. Lab / `data-next` vaults. Never import CT101 into workstation next ([UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md)).  
**Glossary:** piece / artifact / fixture / assembly — [little-tools-lab CONTEXT.md](../../little-tools-lab/CONTEXT.md).

**Scaffold order:** `forget-lint` → `verify-gates` → `token-economy`.

> **Deep briefs (SoT for algorithms, schemas, full inventories):**  
> [forget-lint-primary-sources-2026-07-20.md](./forget-lint-primary-sources-2026-07-20.md) ·  
> [verify-gates-research-brief-2026-07-20.md](./verify-gates-research-brief-2026-07-20.md) ·  
> [token-economy-primary-sources-2026-07-20.md](./token-economy-primary-sources-2026-07-20.md)  
> This file is the **cross-tool synthesis** only — do not duplicate brief bodies here.

---

## Subsystem atlas

```text
ingest / session_distill / dream / spark
        ↓ extract → prepare → verify (end-of-pipeline today: KgPromoter)
  semantic_vault  ──confidence < 0.85──►  quarantine_vault
        ↓ promote (lifecycle)
     honeypot  (is_latest, supersedes_id, graph_rel, decay_class, recall_count)
        ├─ evidence (+ evidence-locate)
        ↓ ripen M5 → knowledge_core → wiki (optional)
```

| Layer | Role for the three tools |
|-------|--------------------------|
| Honeypot + lifecycle | Supersession residue → **forget-lint** |
| Evidence + Quality pieces | Mid-pipeline gates → **verify-gates** |
| Routing / context caps | Pre-call budget advice → **token-economy** |
| CT101 daemon / living | Out of scope |

**CT101 census (SSH 2026-07-20):** vault 61 081 · honeypot latest 38 730 · `is_latest=0` **9 487** · quarantine 1 012 · evidence ~48k.  
**Caveat:** every `is_latest=1` row currently has `recall_count=0` — zero-recall alone cannot drive forget candidates.  
**Caveat:** vault mentions `GZMO_ENABLE_GATES`; **this clone has no such symbols** — verify is end-of-pipeline `KgPromoter` only.

**Tier-1 (do not re-ship):** `honeypot-gate`, `spark-link`, `rem-substrate`, `rrf-recall`, `synapse-tail`, `evidence-locate`, `faithfulness-judge`.

**Lab hot path today:** `cognition-smoke` = session-distill → honeypot-gate → promote → spark-link → rrf-recall → evidence-locate (fixture) → meta.

---

## 1. `forget-lint`

**One-liner:** Active clearing of intermediate / superseded honeypot layers (plan/apply + tombstones); honeypot-gate stays qualify-only.  
**CT101 facts:** `690cb295`, `26ad76d0`, `20811f83`, `f8006497`.  
**Deep brief:** [forget-lint-primary-sources-2026-07-20.md](./forget-lint-primary-sources-2026-07-20.md)

| | |
|--|--|
| **CLI** | `forget-lint plan\|apply --vault … [--dry-run]` |
| **Seams** | Reuse `lifecycle::supersede` / ripen winner `confidence*(1+recall_count)`; do not mutate via honeypot-gate |
| **Must remember** | Prefer age + supersession + Extends siblings; protect Structural; refuse `/opt/gzmo` |
| **Non-goals** | CT101 import; hard-delete default; living GREEN gate; growing gate into a mutator |

**Ready:** sibling piece [`forget-lint`](../../forget-lint/) (plan/apply + fixture smoke) · LTL manifest `partial`.

---

## 2. `verify-gates`

**One-liner:** Analyze / Retrieve / Reason packet gates that call Quality pieces as leaves — not a second RAG.  
**CT101 facts:** `ffd73b42`, `11d7748d`, `663533a6`, `b960e95c`, `acb8905d` (+ Tools Are Leaves `2102986b`).  
**Deep brief:** [verify-gates-research-brief-2026-07-20.md](./verify-gates-research-brief-2026-07-20.md)

| | |
|--|--|
| **CLI** | `verify-gates check --stage analyze\|retrieve\|reason --packet …` |
| **Seams** | retrieve → `evidence-locate`; reason → `faithfulness-judge`; core `KgPromoter::verify` stays production end-gate |
| **Must remember** | Retrieve **judges** recall bundles — does not embed/RRF; Evidence-First cites `[E#]` |
| **Non-goals** | Qdrant inside piece; replace overnight `KgPromoter`; CT101 vault import; tool-chaining product |

**Ready:** sibling piece [`verify-gates`](../../verify-gates/) (analyze/retrieve/reason fixture smoke) · LTL manifest `partial`.

---

## 3. `token-economy`

**One-liner:** Advisory pre-call token budget / compress / profile hints (TALE + Co-Saving as research cites). Obolus stays per-watt.  
**CT101 facts:** `06c23921`, `b3be19fc`, `c605279c`, `5d774056`, `9c827e76`.  
**Deep brief:** [token-economy-primary-sources-2026-07-20.md](./token-economy-primary-sources-2026-07-20.md)

| | |
|--|--|
| **CLI** | `token-economy estimate … -o budget.json` (+ optional TOML snippet, human merge) |
| **Seams** | Read `gzmo-next` / spark caps + `context.rs` trim; may *read* IpW advice; never set `blocks_distill` |
| **Must remember** | CT101 cloud routing table is **not** next defaults; dream `max_tokens_*` TOML keys may be unwired |
| **Non-goals** | Graft CT101 cloud routes; re-ship Obolus; auto-edit living/`~/.gzmo` config |

**Ready:** estimator table · budget schema · chat-vs-heavy fixture divergence · `blocks_distill=false` assert.

---

## Assembly sketch (future)

```text
session-distill
  → [verify-gates analyze]          # optional
  → honeypot-gate
  → [forget-lint plan|apply]        # lab vault hygiene
  → promote
  → [token-economy estimate]        # advisory before LLM-heavy
  → spark-link → rrf-recall
  → [verify-gates retrieve|reason]  # optional
  → evidence-locate batch → meta
```

Adding pieces needs an explicit little-tools catalog PR (closed set of 46 today).

---

## Provenance

| Kind | Source |
|------|--------|
| Deep briefs | Linked above (explore agents 2026-07-20) |
| CT101 | `/opt/gzmo/data/vault.db` read-only SSH |
| Code | `gzmo-core` memory/{ripen,lifecycle,honeypot,vault,kg_extract}, config/gateway/context; Tier-1 piece CONTEXT files |
| Prior dig | [ct101-vault-archaeology-2026-07-20.md](./ct101-vault-archaeology-2026-07-20.md) |
