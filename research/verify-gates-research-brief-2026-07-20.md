# Research brief: `verify-gates` (mid-pipeline Analyze / Retrieve / Reason)

**Date:** 2026-07-20  
**Status:** primary-source inventory for a future little-tool / cognition piece  
**Boundary:** CT101 vault is evidence only — do not import into `data-next` ([`docs/UNIQUENESS_THESIS.md`](../docs/UNIQUENESS_THESIS.md); archaeology note).  
**Catalog shape (prior pick):** `verify-gates check --stage analyze|retrieve|reason --packet …` ([`research/ct101-vault-archaeology-2026-07-20.md`](ct101-vault-archaeology-2026-07-20.md) § Executive picks #2).

---

## 1. Problem statement (CT101 fact ids)

Living vault: `/opt/gzmo/data/vault.db` on CT101 (read-only SQLite). All six requested ids are `CuratedVault`, `confidence=1.0`, `source_file=drive-research-tinyfolder-gzmo-architecture-analysis-product.md`, `created_at≈2026-06-08T22:59Z`.

| Fact id (prefix / full) | Content (verbatim) | Role for `verify-gates` |
|-------------------------|--------------------|-------------------------|
| `ffd73b42` / `ffd73b42-8cf3-4806-a72f-22e55475234a` | `[PROJECT:Intermediate Verification Gates] Addresses the 'Verification is only at the end of the pipeline' gap.` | **Product name** — mid-pipeline gates close the end-only gap. |
| `11d7748d` / `11d7748d-6db1-4f0d-9db4-b1f1bb89442b` | `[CONCEPT:Verification is only at the end of the pipeline] No per-node gates (Analyze Gate, Retrieve Gate, Reason Gate).` | **Gap definition** — three named stages. |
| `663533a6` / `663533a6-cf8f-45a1-ba57-63ae06835736` | `[CONCEPT:Verification is only at the end of the pipeline] Partial implementations exist but are disabled by default.` | **Scaffold claim** — research asserts stubs exist; see caveat below. |
| `b960e95c` / `b960e95c-c80d-4a7a-9089-d6ce5a6cd7fc` | `[CONCEPT:Evidence-First RAG] Each action must cite an [E#] from a provided evidence package.` | **Contract** — claims must cite packaged evidence ids. |
| `acb8905d` / `acb8905d-e4cf-4f5a-a66f-a13ca422f78f` | `[CONCEPT:Evidence-First RAG] Enforces citations and blocks unsafe path claims.` | **Fail-closed** — block hallucinated / unsafe paths. |
| `2102986b` / `2102986b-e6a7-4c60-9c46-23280063bd8a` | `[CONCEPT:Tools Are Leaves] A gap identified in architectural research.` | **Adjacent gap** — tools don’t chain; gates must call siblings as leaves via CLI/files, not become a second retrieval organ. |

**Supporting CT101 facts (same source cluster, not in the requested set):**

| Id prefix | Content |
|-----------|---------|
| `a617cbb0` | `[PROJECT:Intermediate Verification Gates] Wire analyzeGate, retrieveGate, reasonGate into the main pipeline.` |
| `e3962612` | `[PROJECT:Intermediate Verification Gates] A mid-term strategic recommendation.` |
| `385179d2` | `[RELATION:PROJECT] Intermediate Verification Gates → Verification is only at the end of the pipeline` |
| `a2b1959d` | `[CONCEPT:Evidence-First RAG] Safety Verifier blocks hallucinated paths.` |
| `bdd0be5b` | `[CONCEPT:Tools Are Leaves] No tool chaining where vault_read(A) finds a reference to B -> Auto-read B.` |

**Pi / curated source (same tinyFolder product analysis):** German product note states there are no per-node Analyze / Retrieve / Reason gates to catch errors early; gaps 5–7 have partial implementations (`GZMO_ENABLE_GATES`, `GZMO_ENABLE_TOOL_CHAINING`, `GZMO_ENABLE_MODEL_ROUTING`) disabled by default — “the scaffolding exists; the loop is not closed” (Qdrant knowledge chunk `drive-research-tinyfolder-gzmo-architecture-analysis-product.md#chunk26`, via `gzmo_memory_search`).

**Caveat (local GZMO tree):** Grep of `/home/gzmo/github-clone/GZMO` finds **no** `GZMO_ENABLE_GATES`, `analyzeGate`, `retrieveGate`, or `reasonGate` symbols. The “partial implementation” claim is **corpus/research evidence**, not a present feature flag in this clone. What *does* exist today is **end-of-pipeline** verify (`KgPromoter::verify`) after extract/merge — which is exactly the gap the project names.

**Identity alignment:** Canonical pipeline is already “extract → verify → promote → vault → honeypot” ([`MACHINE.md`](../MACHINE.md)). The problem is **placement**: verify runs once at the end of extract (and similar once in dream/spark/distill), not as named mid-stage gates that can fail closed before expensive downstream work.

---

## 2. Primary-source inventory — current end-of-pipeline verify

### 2.1 Shared KG extract → verify → promote (`gzmo-core`)

| Path | Symbols | What it owns |
|------|---------|--------------|
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/memory/kg_extract.rs` | `KgGateConfig`, `KgPromoter`, `run_pipeline`, `run_merged_pipeline`, `extract`, `verify`, `apply_verdicts`, `verdict_passes`, `promote_to_kg`, `extraction_schema`, `verification_schema`, `Verdict`, `VerificationResult`, `VerifiedEntity`, `VerifiedRelation` | **Canonical end-of-pipeline verify.** LLM fact-checker over numbered E/R candidates against SOURCE; drops unsupported / low-confidence / short-evidence. |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/memory/kg_promotion.rs` | `MIN_EVIDENCE_CHARS` (= 12), entity/relation validators | Evidence length floor used by `verdict_passes`. |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/memory/evidence_localize.rs` | `localize_evidence`, `localize_observation_evidence` | Post-verify quote → `EvidenceSpan` (±1 sentence window). |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/types.rs` | `EvidenceSpan`, `ExtractedTruth` | Persisted span shape (`evidence_text`, `quote_verifier`, `char_start`/`char_end`). |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/ingest.rs` | `IngestEngine`, `new_with_verify`, `run_pipeline`, `truths_from_pipeline`, `collect_truths` | Document ingest: extract-per-chunk → merge → **verify once** → localize → promote vault/KG. |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/dreams.rs` | DreamEngine + `with_verify_gateway` | Same `KgPromoter` verify path for dreams. |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/session_distill.rs` | SessionDistillEngine + verify gateway | Same promoter for chat distill. |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/spark.rs` | SparkEngine verify gateway routing (`"verify"`) | Separate verify gateway for spark. |
| `/home/gzmo/github-clone/GZMO/gzmo-core/src/config.rs` | `kg_gate()` on ingest/dream/distill configs | Defaults: `verify=true`, `min_confidence=0.85`, `require_evidence=true`, `strict_kg=true`, `verify_temperature=0.1`. |

**Verify pass criteria (code):** `Verdict.supported == true` **and** `confidence >= gate.min_confidence` **and** (if `require_evidence`) `evidence.trim().len() >= MIN_EVIDENCE_CHARS` ([`kg_extract.rs`](../gzmo-core/src/memory/kg_extract.rs) `verdict_passes`; [`kg_promotion.rs`](../gzmo-core/src/memory/kg_promotion.rs)).

**Verify I/O shape (LLM structured JSON):**

```json
{
  "entity_verdicts": [
    { "index": 0, "supported": true, "confidence": 0.9, "evidence": "≥12 char quote" }
  ],
  "relation_verdicts": [ /* same */ ]
}
```

(`verification_schema` / `Verdict` in `kg_extract.rs`.)

### 2.2 Sibling Quality pieces (already extracted)

| Path | Symbols / CLI | Role |
|------|---------------|------|
| `/home/gzmo/github-clone/evidence-locate/` | `localize_evidence`, `localize_observation_evidence`, `EvidenceSpan`; CLI `locate`, `batch` | Deterministic quote localization (port of `evidence_localize.rs`). **No LLM, no RAG.** |
| `/home/gzmo/github-clone/faithfulness-judge/` | `judge_claim`, `evidence_in_source`, `run_probes`; CLI `run`, `single` | Claim support vs context/corpus (proxy word-overlap or optional LLM). **No retrieval index.** |
| `/home/gzmo/github-clone/etl-cli/` | `extract_heuristic`, `Verifier::verify`, `run_pipeline` | Lab-only heuristic extract+vague-phrase gate; **demoted / superseded by session-distill** ([`PIECE_ELEVATION_MAP.md`](../../little-tools-lab/docs/PIECE_ELEVATION_MAP.md)). |
| `/home/gzmo/github-clone/GZMO/scripts/ingest-quality/faithfulness-judge.py` | (upstream of faithfulness-judge piece) | CT101 ingest-quality gate scripts. |

### 2.3 Lab assembly / registry surfaces

| Path | Role |
|------|------|
| `/home/gzmo/github-clone/little-tools-lab/scripts/cognition-smoke.sh` | Stages 1–5 cognition chain; **Stage 6** already runs `evidence-locate batch` as quality floor; meta via `cognition-pipeline-meta`. |
| `/home/gzmo/github-clone/little-tools-lab/common/src/pipeline/types.rs` | `PipelineFact`, `FactStatus::{Extracted,Qualified,Hypothesized,Verified,Promoted,Rejected}`, `EvidenceSpanRecord` |
| `/home/gzmo/github-clone/little-tools-lab/common/src/pipeline/registry.rs` | `attach_evidence`, `promote` (Verified + confidence threshold) |
| `/home/gzmo/github-clone/little-tools-lab/CONTEXT.md` | Piece / assembly / fixture-smoke vocabulary |
| `/home/gzmo/github-clone/little-tools-lab/catalog/ASSEMBLIES.md` | Recipe index: cognition-smoke, ingest-smoke, etc. |
| `/home/gzmo/github-clone/little-tools-lab/scripts/ingest-smoke.sh` | Batch ingest via `gzmo ingest-dir` (live) — end-of-pipeline verify stays inside core ingest |

---

## 3. Three stage contracts (analyze / retrieve / reason)

Design target from CT101 naming (`11d7748d`, `a617cbb0`) + Evidence-First (`b960e95c`, `acb8905d`) + existing JSON artifacts (not inventing a parallel RAG API).

### 3.0 Shared envelope

CLI: `verify-gates check --stage <analyze|retrieve|reason> --packet <path.json> [-o verdict.json]`

Proposed packet / verdict (aligned with `KgPromoter` verdict fields + `EvidenceSpan` + `PipelineFact`):

```json
{
  "schema": "verify-gates.packet.v1",
  "stage": "analyze|retrieve|reason",
  "packet_id": "uuid",
  "source": {
    "kind": "document|session|recall_bundle|hypothesis",
    "uri": "path or logical id",
    "body_path": "optional path to full text"
  },
  "evidence_package": [
    { "id": "E1", "text": "…", "char_start": 0, "char_end": 42, "origin": "source|retrieve|prior_gate" }
  ],
  "claims": [
    {
      "id": "C1",
      "text": "…",
      "cite": ["E1"],
      "kind": "observation|entity|relation|path|hypothesis"
    }
  ],
  "stage_payload": { }
}
```

**Verdict artifact:**

```json
{
  "schema": "verify-gates.verdict.v1",
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
      "span": {
        "evidence_text": "…",
        "quote_verifier": "…",
        "char_start": 16,
        "char_end": 112
      },
      "rejection_reasons": []
    }
  ],
  "stats": { "checked": 1, "passed": 1, "dropped": 0 }
}
```

**Global fail-closed rules (all stages):**

1. Every claim must list ≥1 `cite` → existing `evidence_package[].id` (`b960e95c`).  
2. Missing / unknown `E#` → fail claim (`acb8905d`).  
3. Path-like claims that cite no localized span when `body_path` present → fail (`acb8905d`, `a2b1959d`).  
4. Confidence floor default **0.85** (match `KgGateConfig::default`).  
5. Quotable evidence length ≥ **12** chars when a quote is required (match `MIN_EVIDENCE_CHARS`).

### 3.1 Analyze gate

| | Contract |
|--|----------|
| **When** | After prep / chunking / candidate extract, **before** promote or expensive multi-hop work. Maps to “is the source analysis / candidate set well-formed?” |
| **`stage_payload`** | `{ "candidates": [ { "index": 0, "name": "…", "type": "…", "observations": ["…"] } ], "relations": […], "doc_class": "…" }` — same conceptual objects as `KgEntity` / `KgRelation`. |
| **Pass** | Candidates pass noise filters analogous to `filter_noise_entities` / `prepare_candidates`; each kept claim cites an `E#` whose text appears in `source.body` (via evidence-locate); optional faithfulness proxy on observations vs body. |
| **Fail** | Empty candidates when extract claimed success; vague-only set (etl-cli `is_vague` patterns as soft signal only); cite without localizable span when `require_span=true`; confidence &lt; floor. |
| **Does not** | Call Qdrant/Neo4j; rewrite extract prompts; replace `KgPromoter::extract`. |

### 3.2 Retrieve gate

| | Contract |
|--|----------|
| **When** | After a **caller-provided** recall / evidence bundle is attached (rrf-recall, vault search, session scratch) — gate judges the **bundle**, it does not retrieve. |
| **`stage_payload`** | `{ "query": "…", "hits": [ { "id": "…", "text": "…", "score": 0.1, "source": "…" } ], "budget": { "max_hits": 10 } }` |
| **Pass** | Every hit promoted into `evidence_package` with stable `E#`; every downstream claim cites only those ids; optional diversity / non-empty checks; faithfulness-judge `mode=context` or `either` against hit texts. |
| **Fail** | Claims cite ids outside package; empty hits when stage marked required; unsafe path claims in hit text without citation discipline (`acb8905d`). |
| **Does not** | Embed, RRF, or open collections — **no second RAG stack** (`2102986b` / Tools Are Leaves: compose `rrf-recall` upstream). |

### 3.3 Reason gate

| | Contract |
|--|----------|
| **When** | After hypothesis / relation / spark / answer draft that must be grounded in the evidence package (maps closest to today’s `KgPromoter::verify` + spark verify). |
| **`stage_payload`** | `{ "hypothesis": "…", "steps": ["…"], "answer": "…" }` or KG candidate list for verify. |
| **Pass** | Each reasoning claim entailed by cited `E#` texts: faithfulness-judge (dry-run default; `--live` optional LLM); evidence-locate localizes quotes into spans; mirrors `verdict_passes`. |
| **Fail** | Unsupported / low confidence / empty evidence; relation without directed support in package (same rule as verify system prompt rule 5). |
| **Does not** | Own the reasoner LLM; only judges packets. Optional `--live` may call the same OpenAI-compatible endpoint faithfulness-judge already uses — still not a retrieval stack. |

---

## 4. Calling evidence-locate / faithfulness-judge without a second RAG stack

**Principle:** `verify-gates` is a **packet judge + orchestrator of Quality CLIs**. Retrieval stays in `rrf-recall` / vault MCP / core recall. Localization and faithfulness stay in sibling pieces (CONTEXT: pieces do not library-import each other — file/CLI seams only).

| Concern | Owner | How `verify-gates` uses it |
|---------|-------|----------------------------|
| Quote → char span | `evidence-locate locate --body … --quote …` → `EvidenceSpan` JSON | Analyze + Reason: prove cite text sits in source; attach span to verdict. Batch fixture mode for CI. |
| Claim entailed by context | `faithfulness-judge single --claim … --context …` or `run --probes` | Reason (+ optional Analyze): `supported`/`confidence`/`evidence`; dry-run default (substring proxy). |
| Hybrid recall | `rrf-recall` (cognition-smoke stage 5) | **Upstream of Retrieve gate** — hits become `stage_payload.hits` / `evidence_package`. |
| End-of-pipeline KG verify | `KgPromoter::verify` in core | Remains authoritative inside ingest/dream/distill binaries; `verify-gates` is the **lab/assembly mid-gate** that can later wrap or duplicate the *contract*, not a competing vault index. |
| Registry | `ltl_common::pipeline::PipelineRegistry::attach_evidence` | Verdict spans feed `EvidenceSpanRecord` without new storage plane. |

**Anti-patterns to reject:**

- Embedding or Qdrant client inside `verify-gates`.  
- Re-implementing `localize_evidence` (already ported to evidence-locate; core keeps its copy for ingest).  
- Replacing session-distill / ingest with etl-cli heuristics.  
- Silent auto-`vault_read` chaining (`2102986b` / `bdd0be5b`) — if chaining is needed, that is the separate `tool-chain` candidate from archaeology, not this piece.

---

## 5. Assembly insertion point (etl / cognition-smoke / ingest)

```text
MACHINE (core):
  Any input → prep → extract → verify → promote → vault → qualify → honeypot

Lab today:
  cognition-smoke:
    distill → export → honeypot-gate → promote → spark-link → rrf-recall → evidence-locate(batch) → meta
  ingest-smoke:
    gzmo ingest-dir  (verify inside KgPromoter)
  etl-cli:
    lab-only / demoted heuristic ETL (not production seam)
```

**Recommended insertion (GZMO-next recipes, not CT101 graft):**

| Gate | Insert relative to | Rationale |
|------|--------------------|-----------|
| **analyze** | After session-distill `export` / before or inside honeypot-gate input; **or** after ingest extract candidates if a lab ingest recipe exposes packets | Catches bad candidates early (`11d7748d`). |
| **retrieve** | Immediately **after** cognition-smoke stage 5 `rrf-recall`, **before** treating recall as trusted context for spark/answer | Judges the bundle; evidence-locate stage 6 today is fixture-only quality floor — Retrieve gate should consume **live recall JSON**, not only `cases.json`. |
| **reason** | After spark-link hypothesis (stage 4) and/or before vault promote (3b) when claims must be grounded | Closest to end-of-pipeline verify; can share pass criteria with `KgGateConfig`. |

**etl-cli:** Do **not** make etl-cli the host. Elevation map marks it DEMOTE / lab-only / superseded by session-distill. Use it only as a fixture generator for vague-phrase negatives if useful.

**ingest:** Keep production verify in `IngestEngine` / `KgPromoter`. Optional later: emit `verify-gates` packets from ingest dry-run for beat-gate parity — without changing CT101 cron.

**Closed-set note:** Little Tools Lab is a closed set of 46 pieces ([`CONTEXT.md`](../../little-tools-lab/CONTEXT.md)). Scaffolding `verify-gates` is an **exceptional 47th** unless it replaces/absorbs scope — archaeology already flagged it as “new adjacent”; justify explicitly in catalog PR.

---

## 6. Fixtures / smoke plan

### 6.1 Piece fixture-smoke (S0)

| Fixture | Stage | Expect |
|---------|-------|--------|
| `fixtures/analyze/pass.packet.json` | analyze | `passed=true`; spans non-null for cited quotes |
| `fixtures/analyze/fail-missing-cite.packet.json` | analyze | `passed=false`; rejection mentions missing `E#` |
| `fixtures/retrieve/pass.packet.json` | retrieve | Hits → package; claims only cite package ids |
| `fixtures/retrieve/fail-foreign-cite.packet.json` | retrieve | Cite `E99` not in package → fail |
| `fixtures/reason/pass.packet.json` | reason | Faithfulness dry-run support + evidence-locate span |
| `fixtures/reason/fail-unsupported.packet.json` | reason | Hallucinated claim vs context → fail |

Reuse bodies/quotes from:

- `/home/gzmo/github-clone/evidence-locate/fixtures/cases.json`  
- `/home/gzmo/github-clone/faithfulness-judge/fixtures/probes.yaml` + `context.md`

**Smoke command (proposed):**  
`verify-gates check --stage analyze --packet fixtures/analyze/pass.packet.json`  
(and a `batch --fixture fixtures/suite.json` that asserts all pass/fail expectations).

### 6.2 Assembly fixture-smoke

1. Extend `cognition-smoke.sh` with optional Stage 5b/6b: build retrieve/reason packets from `RECALL_REPORT` + spark report; run `verify-gates` when binary present; record in meta.  
2. Keep Stage 6 evidence-locate batch as dependency check (piece still green alone).  
3. Schema: add `verify-gates` fields to `schemas/cognition-smoke-meta.json` (or nested under `quality`).

### 6.3 Live-smoke (S1, optional)

- `--live` reason gate → faithfulness-judge `--live` against localhost LLM (same as piece README).  
- Do not require Qdrant inside verify-gates; if retrieve live-smoke is needed, feed hits from live `rrf-recall` / vault search **as files**.

### 6.4 Beat-gate (S2)

- Compare mid-gate drop rates vs end-only `KgPromoter` verify on a **copied** vault / golden ingest set (`scripts/ingest-quality/`), not CT101 production writes.

---

## 7. Explicit non-goals

1. **Not a second RAG stack** — no embeddings client, no Qdrant collection, no hybrid ranker inside the piece.  
2. **Not CT101 daemon graft** — lab / GZMO-next assembly only; CT101 remains reference ([`CONTEXT.md`](../../little-tools-lab/CONTEXT.md) CT101 vs GZMO-next).  
3. **Not vault import** from CT101 60k facts ([archaeology](ct101-vault-archaeology-2026-07-20.md) boundary).  
4. **Not replacing `KgPromoter::verify`** in core ingest/dream/distill on day one — contract parity first; core merge is a later elevation decision.  
5. **Not reviving etl-cli as production ETL** ([`PIECE_ELEVATION_MAP.md`](../../little-tools-lab/docs/PIECE_ELEVATION_MAP.md)).  
6. **Not implementing Tool Chaining / `GZMO_ENABLE_TOOL_CHAINING`** (`2102986b` family) — separate archaeology candidate `tool-chain`.  
7. **Not implementing Multi-Model Routing flags** (`GZMO_FAST_MODEL` / judge model project) — Obolus / router already split extract vs verify gateways.  
8. **Not a chat UX or Observatory UI** — CLI + JSON artifacts only (piece convention).  
9. **Not silently counting fixture evidence-locate batch as Retrieve-gate coverage** — today’s cognition-smoke stage 6 does not judge live recall packets.  
10. **Not expanding the closed set without explicit justification** — 47th piece needs catalog + CONTRIBUTING exception.

---

## Provenance

| Source | Use |
|--------|-----|
| SSH CT101 `sqlite3 -readonly /opt/gzmo/data/vault.db` | Verbatim fact texts for six ids + related |
| `user-gzmo-living` `gzmo_memory_search` | Honeypot hits + Pi chunk on `GZMO_ENABLE_GATES` |
| `research/ct101-vault-archaeology-2026-07-20.md` | Product pick, V/F/N, shape |
| `gzmo-core` kg_extract / ingest / evidence_localize / types / config | End-of-pipeline verify truth |
| `evidence-locate`, `faithfulness-judge`, `etl-cli` repos | Sibling contracts |
| `little-tools-lab` CONTEXT, ASSEMBLIES, cognition-smoke, PIECE_ELEVATION_MAP, ltl-common pipeline | Assembly insertion |
| Local grep | Confirmed absence of `GZMO_ENABLE_GATES` / `*Gate` symbols in GZMO clone |

---

## Suggested next build step (out of scope for this brief)

Scaffold piece repo in temp-bench shape: clap `check`/`batch`, packet schema, fixture suite calling evidence-locate + faithfulness-judge subprocesses, then optional cognition-smoke stage hook behind a feature flag / binary presence check.
