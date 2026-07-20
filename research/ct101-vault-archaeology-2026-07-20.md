# CT101 Vault Archaeology — 2026-07-20

**Scope:** Mine living CT101 vault for buried product ideas and serendipitous artefacts.  
**Vault:** `/opt/gzmo/data/vault.db` on CT101 (read-only dig via SSH + `gzmo-living` MCP).  
**Boundary:** Cite CT101 evidence only. Do **not** import this vault into `data-next` ([UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md)).

---

## Snapshot

| Layer | Count |
|-------|------:|
| `semantic_vault` | 61 081 |
| `honeypot` (latest) | 38 730 |
| `honeypot` (all) | 48 217 |
| `evidence` | 48 014 |
| `quarantine_vault` | 1 012 |

**Decay mix (vault):** CuratedVault 57 565 · SessionDistill 2 734 · Episodic 399 · Structural 193 · Procedural 185 · Core 4  

**Honeypot origin (latest):** ingest 34 250 · verified_dream 2 767 · session_distill 1 522 · manual 191  

**Container:** all latest honeypot tagged `obolus`.

**High-signal source clusters (latest honeypot):** daily `memory/*.md`, pi-mentor discovery sessions, `drive-research-tinyfolder-gzmo-architecture-analysis-product.md` (61), `the-sovereign-software-factory-blueprint.md` (169), cascading/honeypot/wisdom docs, Obolus micros, `manual/core_insight_20260717.md` (104), Phantom Drive micros, agentic token-economy blueprint.

`01_Ideas_Inbox` exists only as a **synthetic wiki stub** + research facts — not a live folder of product briefs on CT101.

---

## Executive picks (build / extract next)

Three candidates that are **new relative to the Little Tools catalog**, with strong CT101 evidence and a clear little-tool or product shape. Already-extracted Tier-1 IP (spark-link, honeypot-gate, rem-substrate, …) is listed under *Already cataloged* below — do not re-ship.

> **Shipped lab siblings (2026-07-20):** `forget-lint` · `verify-gates` · `token-economy` · `tool-chain` · `trace-memory` (LTL `--frontier`).  
> **learning-loop-A (#6):** deepen recipes — see [LEARNING_LOOP_A.md](../docs/LEARNING_LOOP_A.md); cognition-smoke vault spark defaults to `--anchor-window 0,30` for young `data-next`.

### 1. `forget-lint` — Purposeful forgetting / intermediate-honeypot clearer

**One-liner:** Active decay CLI that clears intermediate / superseded honeypot layers (and optional quarantine promote/demote), treating forgetting as a first-class product feature rather than passive half-life.

| | |
|--|--|
| **Evidence** | `690cb295-…`, `26ad76d0-…` (Purposeful Forgetting); `20811f83-…` (Lint as purposeful forgetting); `f8006497-…` (clearing intermediate honeypots); sources: `the-cascading-honeypot-theorem-of-wisdom.md`, `the-honeypot-compiler-architecture-distilling-dat.md` |
| **vs catalog** | **New** — honeypot-gate qualifies/classifies; nothing ships *forgetting as product* |
| **V / F / N** | 9 / 7 / 9 |
| **Shape** | little-tool CLI: `forget-lint plan\|apply --vault … --dry-run` → JSON of candidates + tombstones |
| **Boundary** | Operate on lab/`data-next` vault copies; never bulk-import CT101 |

### 2. `verify-gates` — Intermediate Verification Gates (per-node pipeline gates)

**One-liner:** Analyze / Retrieve / Reason gates **inside** the pipeline (not only end-of-pipeline verify), closing the “verification only at the end” gap from the tinyFolder product analysis.

| | |
|--|--|
| **Evidence** | `ffd73b42-…` (PROJECT: Intermediate Verification Gates); `11d7748d-…`, `663533a6-…` (end-only verification gap); `b960e95c-…`, `acb8905d-…` (Evidence-First RAG); source: `drive-research-tinyfolder-gzmo-architecture-analysis-product.md` |
| **vs catalog** | **New adjacent** — etl-cli / faithfulness-judge / evidence-locate cover extract→verify→promote and quote localization; **not** mid-pipeline node gates |
| **V / F / N** | 8 / 6 / 8 |
| **Shape** | little-tool or cognition-piece: `verify-gates check --stage analyze\|retrieve\|reason --packet …` |
| **Boundary** | Wire into gzmo-next assembly recipes; cite CT101 gaps only |

### 3. `token-economy` — Reactive token budget / Co-Saving router

**One-liner:** Budget, route, compress, and cache context as an explicit economy (TALE-style estimator + Co-Saving patterns), exposed as a CLI that emits routing/TOML snippets for GZMO cognition jobs.

| | |
|--|--|
| **Evidence** | `06c23921-…`, `b3be19fc-…`, `c605279c-…` (Reactive Token Economy); `5d774056-…` (Co-Saving ~50.85%); `9c827e76-…` (TALE); sources: `drive-research-agentic-token-economy-blueprint-micro0{1,2,3}.md` |
| **vs catalog** | **New** — Obolus answers “smart per watt”; this answers “smart per token/context” |
| **V / F / N** | 8 / 5 / 8 |
| **Shape** | little-tool CLI → `budget.json` + optional `[routing]` / context-pack hints |
| **Boundary** | Research → lab prototype; do not graft CT101 routing table wholesale |

**Honorable mention (product narrative, not a tool):** treat GZMO publicly as a **Knowledge Operating System / Sovereign Reasoning Engine** (`026f014e-…`, `92b1985a-…`, `32a14d01-…`, `6eaf665f-…`, `35652172-…`) — positioning already matches UNIQUENESS_THESIS; ship as docs/positioning, not another daemon.

---

## Full scored catalog

Scores: Value / Feasibility / Novelty (1–10). Novelty vs Little Tools catalog + shipped repos under `/home/gzmo/github-clone/`.

| # | Candidate | One-liner | Evidence (fact ids / sources) | vs catalog | V | F | N | Suggested shape |
|--:|-----------|-----------|-------------------------------|------------|--:|--:|--:|-----------------|
| 1 | `forget-lint` | Purposeful forgetting of intermediate honeypots | `690cb295`, `26ad76d0`, `20811f83`; cascading/wisdom + honeypot-compiler | **new** | 9 | 7 | 9 | little-tool CLI |
| 2 | `verify-gates` | Mid-pipeline Analyze/Retrieve/Reason gates | `ffd73b42`, `11d7748d`, `b960e95c`; tinyfolder product analysis | **new** | 8 | 6 | 8 | little-tool / assembly piece |
| 3 | `token-economy` | Reactive budget/route/compress/cache | `06c23921`, `5d774056`, `9c827e76`; agentic-token-economy micros | **new** | 8 | 5 | 8 | little-tool CLI |
| 4 | `tool-chain` | Close “Tools Are Leaves” — follow refs → auto-read | `2102986b`, `bdd0be5b`, `af0999bb`; tinyfolder product | **scaffolded** — sibling + LTL `--frontier` | 7 | 6 | 7 | little-tool CLI `expand` |
| 5 | `trace-memory` | Cross-task trace retrieve → inject strategies | `9a3763c7`; tinyfolder product | **scaffolded** — sibling + LTL `--frontier` | 7 | 6 | 7 | little-tool CLI `recall`/`record` |
| 6 | `learning-loop-A` | Close Phase A learning loop (highest impact / lowest risk per research) | `75eb5004`, `6eaf665f`; tinyfolder product | **deepening** — fixture `phase_a_proof` + `learning_loop` meta; live LLM `promoted:true` still open | 9 | 5 | 6 | deepen dream/spark wiring (not new organ) |
| 7 | `pdu-reflect` | Prosecutor–Defender–Umpire Hegelian reflection | `190d23e6`, `c59bdcd9`, `f13f17b9`; how-could-we-blueprint-an-idea | **new** (zpd-tutor is pedagogy, not PDU) | 6 | 5 | 8 | little-tool / lab session |
| 8 | `stigmergy-queue` | Ideas→Architecture→Build→QA folder pipeline | `a4949023`, `54aec32d`, `d1473f8e`, `a5e885a4`; du-hast-gesagt + sovereign factory | **new as product** | 6 | 4 | 6 | research / optional Obsidian skill — not CT101 import |
| 9 | `ipw-dashboard` | Intelligence-per-Watt IDE status + hot-swap | `d2446b2a`, `2d7b181e`, `afe212a4`; Obolus VS Codium research | **partial** — Obolus shipped; IDE extension not | 7 | 4 | 6 | VSCodium extension (separate from little-tools) |
| 10 | `phantom-drive` | Air-gapped USB sterile LLM + mountpoint watchdog | `4ccab818`, `62bed1b2`, `9ddb4fa0`; phantom-drive micros | **new** | 5 | 3 | 9 | research / hardware product — out of little-tools scope |
| 11 | `cascading-compiler` | LLM-Compiler → Executable Wisdom wiki cascade | `f36d8361`, `21eab4a8`, `da4ee01f`; cascading honeypot sources + wiki entities | **identity adjacent** — wiki emit exists; full cascade undersold | 8 | 4 | 7 | positioning + deepen wiki emit (not vault import) |
| 12 | `kos-positioning` | Knowledge OS / Sovereign Reasoning Engine narrative | `026f014e`, `92b1985a`, `32a14d01`, `35652172`; tinyfolder + core_insight | **aligned with thesis** | 9 | 9 | 5 | docs / MACHINE identity only |
| 13 | spark / honeypot / rem / rrf / synapse | Serendipity + gate + REM + hybrid recall + audit bus | CT101 implements; catalog Tier 1 | **already extracted** | — | — | — | skip re-ship |
| 14 | Obolus / temp-bench / evidence-locate | IPW bench, temp sweep, quote windows | shipped repos | **already shipped** | — | — | — | skip |

---

## Spark / serendipity artefacts

LLM-free pairing: stale `CuratedVault` anchors (recall_count=0, age ∈ [14,90]d, product-ish sources) × recent session_distill / verified_dream / manual (≤21d), scored with triangular `stale_sweetness` + lexical overlap + concept-tag bridges. **Hypotheses only — not promoted facts.**

| # | Score | Stale anchor | Recent fact | Hypothesized link |
|--:|------:|--------------|-------------|-------------------|
| 1 | 0.51 | Obsidian Vault as stigmergic memory broker (`9b8a5972-…`, du-hast-gesagt) | SessionDistill extracts chats → vault (`8bb31be0-…`, pi-mentor discovery) | Stigmergy folders and SessionDistill are the **same product seam**: markdown drops vs chat JSON both feed curated memory — a `stigmergy-ingest` bridge would unify them |
| 2 | 0.48 | Memory Broker / Sovereign_Vault (`d5a17b51-…`, sovereign factory) | same SessionDistill fact | Sovereign Factory “Memory Broker” is the **product name** for what CT101 already runs as distill→honeypot — package the broker API, don’t rebuild Obsidian queues on next |
| 3 | 0.46 | Executable Wisdom = purest distilled essence (`f36d8361-…`, cascading blueprint) | GZMO.toml defines dreams/spark/ingest/distill (`1765e034-…`) | Executable Wisdom is the **marketing name** for the overnight metabolism already in config — sell the cascade, not a new store |
| 4 | 0.45 | Cascading Honeypot distilled essence (`21eab4a8-…`) | Cognition routed to cloud extract/verify (`53c89529-…`) | Cascade theory assumes local compiler purity; live CT101 routes dream/spark/distill to **cloud verify** — product tension to resolve honestly in positioning |
| 5 | 0.44 | LLM-Wiki maps to Cascading Honeypot (`52f4700e-…`) | `knowledge_core.db` export target for Honeypot Ripen (`fa1f48d5-…`, memory/2026-07-18) | Ripen→knowledge_core is the **engineering toehold** of Executable Wisdom; `forget-lint` + ripen export form the cascade’s write path |
| 6 | 0.44 | Cascading Honeypot vs traditional RAG (`07aa1261-…`) | DreamEngine honeypot REM anchors (`4649563d-…`) | REM substrate is **runtime Cascading Honeypot**: dreams read curated topology, not raw RAG dumps — deepen rem-substrate narrative |
| 7 | 0.43 | LLM-Wiki architecture (`e698aee6-…`) | knowledge_core ripen target (`fa1f48d5-…`) | Wiki emit + ripen export should be one **compiler pipeline** story (ingest → honeypot → knowledge_core → wiki index) |
| 8 | 0.43 | LLM-Compiler architecture (`da4ee01f-…`) | same ripen target | “Honeypot Compiler” research title maps to **ripen + wiki**, not a separate binary — name the existing path |
| 9 | 0.43 | DIKWP×TRIZ→Cascading Honeypot (`36a5160e-…`, wisdom webs) | honeypot REM (`4649563d-…`) | Academic framing (DIKWP/TRIZ) can back a short **paper/HN post** (tinyfolder analysis already suggested this) without new code |
| 10 | 0.43 | TRIZ×DIKWP (`86fdfa56-…`) | honeypot REM | Same artefact cluster as #9 — research packaging, not a little tool |

### Extra high-value cross-links (curated, not from lexical scorer)

- **Wrong product shape** (`35652172-…`, manual core insight) ↔ **Knowledge OS** (`026f014e-…`): identity sentence already in vault; keep next focused on honeypot+verify+promote felt lifecycle (thesis live gap).
- **Phase A Learning Loop** (`75eb5004-…`) ↔ live distill/dream/spark jobs: highest-impact *closure* is wiring + measurement, not a 47th organ.
- **Obolus IPW diagnostics** (`afe212a4-…`) ↔ live Prime/local LLM stack: IDE “wakes on this hardware” bench is the missing UX for Obolus numbers.

---

## Already cataloged / shipped (do not re-extract from CT101)

| Piece | Status |
|-------|--------|
| spark-link, honeypot-gate, rem-substrate, rrf-recall, synapse-tail | Extracted / in little-tools-lab |
| temp-bench, Obolus, evidence-locate, cabinet-sim, zpd-tutor | Standalone repos present |
| UNIQUENESS_THESIS closed-set identity | Authoritative for what *not* to invent |

CT101’s 60k vault is an **evidence mine**, not a migration source. Feature-parity by import would fake uniqueness.

---

## Explicit non-findings

- **No live Ideas Inbox folder** on CT101 — only concept facts + wiki stub.
- **Heavy ingest duplicate noise** — arxiv/`thema_*` and repeated OpenClaw research chunks dominate raw FTS; product signal concentrates in ~15 source files listed above.
- **Pedagogy / chaos / PulseLoop** — present in vault; ADR/thesis keep them off overnight production cron unless amended.
- **Generic tool catalogs** (Amazon product skills, EPLAN, Citavi, etc.) — corpus pollution, not GZMO product IP.
- **Lexical spark without theme filters** pairs Obolus diagnostics with sys_janitor — discarded as noise; filtered pass above is the artefact set.

---

## Suggested follow-ups (after you pick)

1. Scaffold **`forget-lint`** or **`verify-gates`** in little-tools-lab (temp-bench shape).  
2. Draft a one-pager **Knowledge OS / Executable Wisdom** positioning that cites these fact ids without promising CT101 vault parity.  
3. Optional: run `spark-link dry-run` against a **copied** CT101 snapshot in lab for richer embeddings-based sparks (still no import into live next).

---

## Dig provenance

- Pass 1: SQL census on CT101 (`decay_class`, `origin`, top `source_file`, product-pattern LIKE sweeps).  
- Pass 2: targeted source clusters + `gzmo-living` memory/wiki search; dedupe vs Little Tools catalog plan + local repo list.  
- Pass 3: `/tmp/spark_archaeology2.py` on CT101 (stale×recent scorer).  
- Report path: `GZMO/research/ct101-vault-archaeology-2026-07-20.md`.
