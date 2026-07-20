# Living output quality — CT101 (2026-07-20)

**Question:** What is the actual output quality of the CT101 living metabolism today, and what should we develop next so Keep (A+C) compounds the vault instead of more lab scaffolding?  
**Method:** Fresh living + product readiness gates; read-only SQLite census on `/opt/gzmo/data/vault.db`; `DREAMS.md` spark/dream sampling; 10-query search panel via living `gzmo memory search`; faithfulness-living + takeaway-recall artifacts.  
**Boundary:** No vault import into workstation `data-next/`; no second overnight writer; product MCP not pointed at living vault.  
**Artifact bundle (local, gitignored):** `data-next/living-quality/` (census, search panel, spark grades, gate copies).

---

## Executive verdict: **MIXED**

| Layer | Grade | One line |
|-------|-------|----------|
| **Ops / appliance (C)** | **PASS** | Living readiness **GREEN** 23/0/0; daemon active; sidecars + Qdrant drift OK |
| **Felt-recall spot checks** | **PASS** | Faithfulness-living **5/5**; takeaway→recall **HIT**; search panel **10/10** |
| **Product A** | **PASS** | Product readiness **GREEN** 10/0/0; release **v0.1.14** fresh |
| **Overnight content quality** | **WEAK→MIXED** | Dream consolidates; spark **promotes** but **anchor diversity collapsed**; honeypot **`recall_count` all zero**; stale contradictory lore still latest |

**Bottom line:** CT101 is a healthy *machine* and a usable *retrieval surface* for curated doctrine facts. It is **not** yet a high-quality *serendipity / usage-feedback* organism. Vault size (~61k / ~38k honeypot) overstates content quality.

---

## Metrics scorecard

| Metric | Value | Source |
|--------|------:|--------|
| Living gate verdict | GREEN (pass=23 fail=0 hold=0) | `data-next/living-readiness/latest.json` (re-run 2026-07-20T14:14Z) |
| Product gate verdict | GREEN (pass=10 fail=0 hold=0) | `data-next/product-readiness/latest.json` |
| `semantic_vault` | 61 081 | CT101 census `/tmp/living-quality-census.json` → `data-next/living-quality/` |
| Honeypot latest / superseded | 38 730 / (total−latest) | same |
| Quarantine | 1 012 | same |
| Qdrant honeypot points | 38 731 (drift δ≈0) | living-readiness health rows |
| Neo4j (MCP summary) | ~13.5k entities / ~65k relations | living-readiness `health:mcp_memory` |
| `verify_pass=1` on latest honeypot | 100% (38 730) | census |
| `recall_count=0` on latest honeypot | **38 730 / 38 730 (100%)** | census `recall_count_latest` |
| Age band [14,90]d latest | 37 455 | census `age_bands_latest` |
| Origins (latest) | ingest 34 250 · verified_dream 2 767 · session_distill 1 522 · manual 191 | census |
| Decay (latest) | CuratedVault 36 162 · SessionDistill 2 377 · Structural 191 | census |
| Faithfulness-living | 5/5 supported | `data-next/faithfulness-living/latest.json` |
| Takeaway→recall | living_hit (same sitting) | `data-next/ct101-takeaway-recall/latest.json` |
| Search panel HIT rate | 10/10 | `data-next/living-quality/living-search-panel.json` |
| Spark sections in `DREAMS.md` | 1 346 | CT101 `/opt/gzmo/DREAMS.md` |
| Spark grades (last 20) | supported 19 · stale-loop 1 | `living-spark-grades.json` |
| Spark anchors (last 50) | **49×** `[SYSTEM:GZMO] four memory layers…` · 1× SessionDistill | CT101 analysis 2026-07-20 |
| Top historical spark anchors | SparkEngine 242 · GZMO 226 · Prime 192 · DreamEngine 182 · SessionDistill 107 | same |
| Last dream cycle | 2026-07-20 01:02Z — 22 vault truths (`verified_dream`), KG entities=9 relations=2 | `journalctl -u gzmo-daemon` + `DREAMS.md` header |
| Last spark cycles | 00:00Z + 03:30Z promoted KG `HYPOTHESIZED_LINK` (one with `supported=false` quarantine tier) | journal |

---

## Sample grades (rubric)

Rubric: **supported** · **thin** · **stale-loop** · **junk**.

### Dream consolidation (2026-07-19 → processed 01:02Z)

- **Grade: supported (ops)** — Real entities with evidence quotes (living-takeaway session, Prime endpoint, SessionDistill, ripen job). Journal: `Batch promoted truths to vault count=22 origin=verified_dream`.
- **Scar:** Consolidation still narrates / retains **stale honeypot lore** elsewhere in spark history (“DreamEngine currently disabled during clean-slate rebuild”) while dream is clearly **enabled and running**. That is contradictory latest-memory pollution, not a green content signal.
- **Ripen:** Dream text notes honeypot_ripen M5 exported **0 rows** to `knowledge_core.db` — cascade “Executable Wisdom” toehold idle.

### Spark (recent)

- **Grade: stale-loop (diversity)** — Last 50 sparks almost all re-anchor the same `[SYSTEM:GZMO] four-layer memory` fact. Historically dominated by meta-SYSTEM anchors (SparkEngine / DreamEngine / SessionDistill). Serendipity is **self-referential stack mythology**, not broad CuratedVault exploration.
- **Grade: supported (mechanics)** — Selection + hypothesis + KG write still fire (`promoted=true kg=1`). Citation-backed path sometimes lands quarantine (`supported=false confidence=0.6`).
- **Operator surface:** `DREAMS.md` is ~36k lines / 1.3k spark sections — overnight “brief” is not brief.

### Distill

- Journal 02:15Z: mix of real distill (4 truths) + skips (no verified entities / dedup). Queue depth 0 — not backlogged.
- Takeaway proof path works when exercised deliberately (`living-takeaway-*` HIT).

### Felt recall

- Doctrine / ADR / identity queries: **strong** (panel + faithfulness).
- Usage feedback: **absent** (`recall_count` never increments → forget-lint / ripen winner scoring cannot use live popularity).

---

## Scars (do not forget)

1. **`recall_count` dead** — every latest honeypot at 0 (census 2026-07-20; same caveat in amp brief earlier today).
2. **Spark anchor monoculture** — four-layer GZMO / engine SYSTEM facts crowd out mid-band CuratedVault diversity despite 37k mid-band rows.
3. **Contradictory stale facts remain latest** — e.g. DreamEngine “disabled / clean-slate” while overnight dream promotes.
4. **knowledge_core ripen idle** — 0-row export; cascading-compiler story is positioning ahead of living emit.
5. **Historical faithfulness_context 0.806** ([CORE_MECHANICS_AUDIT_20260605.md](../docs/CORE_MECHANICS_AUDIT_20260605.md)) — not re-run as full suite this pass; spot CORE_INSIGHT gate is not a substitute.
6. **Lab ≠ living** — Phase A / `--frontier` proofs on `data-next` do not grade CT101 content quality (this research exists because that gap was real).

---

## P0–P3 answers

| Priority | Finding |
|----------|---------|
| **P0** | C is **ops GREEN** — not blocked on sidecars, dual-writer, or health. |
| **P1** | Bottleneck is **content / feedback quality**, not uptime. Spark diversity + recall_count + stale contradiction cleanup. |
| **P2** | Highest leverage: (1) living recall_count wiring, (2) spark denylist/diversity, (3) plan-only forget of contradictory superseded lore, (4) overnight brief compaction. |
| **P3** | Unpark theater/Arena **after** P2 shows movement on spark diversity + recall_count ≠ 0 under real search. |

---

## Develop-next backlog (Keep-first, ≤7)

**Update 2026-07-20 (same day):** organs 1–4 implemented on branch `feat/living-metabolism-organs` — see [living-metabolism-organs-2026-07-20.md](living-metabolism-organs-2026-07-20.md). Remaining after **deploy to CT101** + prove metrics:

1. ~~**Wire `recall_count`**~~ → **Felt Use** (Glance/Cited/Bonded) shipped; prove on living after deploy.
2. ~~**Spark diversity**~~ → **Refractory Field** + soft-pick shipped; prove unique anchors ≫ 1 after N cycles.
3. ~~**Contradiction hygiene (plan-first)**~~ → **Immune Patrol** dry_run plans shipped; human review before any apply.
4. ~~**Overnight operator brief**~~ → **Night Lymph** + status surface shipped.
5. **Living faithfulness widen** — grow claims beyond 5 CORE_INSIGHT needles; optional weekly recall-eval against curated living set.
6. ~~**Ripen / knowledge_core honesty**~~ → diagnosed + fixed: starved recall, not empty core; see [ripen-honesty-2026-07-20.md](ripen-honesty-2026-07-20.md).
7. **Only then Unpark W1 item** — after Felt Use + Refractory show metric movement on CT101.

---

## What not to do next

- More little-tool siblings as a substitute for living quality.
- CT101 → `data-next` vault import.
- Second overnight writer on the workstation.
- Declaring the vault “high quality” because it is large.

---

## Provenance

| Kind | Path / command |
|------|----------------|
| Living gate | `bash scripts/living-readiness-gate.sh` → `data-next/living-readiness/latest.{json,md}` |
| Product gate | `bash scripts/product-readiness-gate.sh` → `data-next/product-readiness/latest.{json,md}` |
| Faithfulness | `data-next/faithfulness-living/latest.json` |
| Takeaway | `data-next/ct101-takeaway-recall/latest.json` |
| Census / panel / grades | `data-next/living-quality/*.json` (from CT101 SSH 2026-07-20) |
| Journal | `journalctl -u gzmo-daemon` on CT101 (dream 01:02Z, spark 00:00/03:30Z) |
| Doctrine | [SPINE_FOCUS.md](../docs/SPINE_FOCUS.md), [LIVING_PRODUCTION_READINESS.md](../docs/LIVING_PRODUCTION_READINESS.md), [ADR-0003](../docs/ADR-0003-one-instance-metabolism.md) |
