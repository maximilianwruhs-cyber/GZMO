# Distillation failure — implementation handoff

**Status:** implemented (2026-07-24) — goal ≥10 CoreCrystallize export-eligible **reached on CT101 (15)**  
**Date:** 2026-07-24  
**Authority scars:** `research/ripen-honesty-2026-07-20.md`, `research/opportunities/felt-use-ripen-floor.md`  
**Living path:** CT101 `gzmo-daemon` only — never workstation `gzmo-serve` overnight  
**Do not:** lower `--min-recall` to fake Brain Feed green

### Shipped in this arc

| Piece | Path | Notes |
|-------|------|-------|
| Claims | `config/core-crystallize/CORE_CLAIMS.toml` | 15 insight claims |
| Crystallize | `scripts/core-crystallize.sh` | enqueue + ensure_land (verbatim `CoreCrystallize:`) + Bonded recall=5 |
| Bonded pin | `gzmo-core/src/memory/core_pin.rs` + vault promote hook | living binary may lag; SQL ensure_land covers until deploy |
| Ingress | `honeypot.rs` reject notebooklm/drive_clean/takeout | path honesty |
| Ripen honesty | banners on `export-knowledge-core.py` / `ripen-knowledge-core.py` | two ripen meanings |

**Operator apply:** `CORE_CRYSTALLIZE_APPLY=1 bash scripts/core-crystallize.sh`  
**Artifact:** `data-next/core-crystallize/latest.{json,md}`

---

## 0. Problem statement (shared context for every slice)

```text
promote → honeypot (recall_count DEFAULT 0)
     → living search often never Felt-Uses those rows
     → dual gate (conf≥0.90 ∧ recall≥3 ∧ origin∈{ingest,verified_dream,session_distill}) empty
     → export-knowledge-core.py emits 0  →  ripen/latest.json advice=starved_recall|origin_filter
     → CORE (knowledge_core fact export) starves while research library stays fat
```

**Intellectual failure (product):** hard-won analysis lives in Pi knowledge / Drive / NotebookLM chunks and never becomes crisp, recall-earning honeypot facts that ripen into CORE.

**Mechanical failure (stack):** Felt Use exists (`felt_use.rs`) but mass of latest honeypot never gets `Cited`/`Bonded`; promote never bumps recall.

| Layer | File | Role |
|-------|------|------|
| Weights | `gzmo-core/src/memory/felt_use.rs` | Glance=+1, Cited=+3, Bonded=+5 |
| SQL bump | `SqliteVault::reinforce_by` in `vault.rs` | `recall_count += delta` |
| Search touch | `platform_memory.rs` | Cited if scratch write, else Glance |
| MCP | `gzmo-core/src/mcp/serve.rs` | `write_scratch` default **true** |
| Qualify | `honeypot.rs` | conf≥0.85, source_file, anti-boilerplate |
| Derived bar | `lifecycle.rs::is_unverified_derived` | dream/spark/session_distill need evidence or conf≥0.92 |
| Distill promote | `session_distill.rs` | origin `"session_distill"` |
| Export | `scripts/export-knowledge-core.py` | living overnight CORE path |
| Census | `scripts/felt-use-depth.sh`, `gzmo ripen status` | measure, don’t gym |

**Sidenote — two “ripen” meanings:** living overnight shells **fact export** (`export-knowledge-core.py`). Charter **concept cards** (`ripen-knowledge-core.py` / Rust `ripen_honeypot`) are a different schema. Do not “fix ripen” by shipping the wrong one.

---

## 1. Close the Felt Use loop (throughput)

### 1.1 Goal

Every living search that *matters* for CORE candidates must call `felt_use::touch` / `touch_hits` on CT101 vault rows so `recall_count` can reach ≥3 without memory-gym spam.

### 1.2 Prove current attach path

**Steps**

1. On workstation (operator):
   ```bash
   bash scripts/felt-use-depth.sh
   GZMO_CONFIG=/opt/gzmo/gzmo.toml  # via ssh
   ssh ct101 'GZMO_CONFIG=/opt/gzmo/gzmo.toml /opt/gzmo/current/target/release/gzmo ripen status'
   ```
2. Note baseline: `latest`, `recall_ge1`, `recall_ge3`, `share_ge3` (of felt), `ripen/latest.json` advice.
3. Run one living MCP search **with** scratch (default):
   - Cursor / OpenClaw MCP `gzmo_memory_search` query that matches known honeypot doctrine facts.
   - Confirm `write_scratch` is not forced false.
4. Re-run census; expect `recall_ge1` and ideally some `recall_ge3` movement on touched ids.

**Code to read first**

```202:208:gzmo-core/src/platform_memory.rs
        // Felt Use: Cited when scratch written, else Glance for ranked hits.
        let kind = if scratch_written {
            FeltUseKind::Cited
        } else {
            FeltUseKind::Glance
        };
        felt_use::touch_hits(&self.vault, items.iter().map(|h| h.fact_id.as_ref()), kind);
```

```153:161:gzmo-core/src/mcp/serve.rs
    // write_scratch unwrap_or(true) — default Cited path
```

**Thoughts / sidenotes**

- **Pi-knowledge hits have `fact_id=None`** → `touch_hits` skips them. Searching only Qdrant library never feeds ripen. Queries must hit **honeypot/vault** rows.
- **Glance needs 3 searches** on the same id to clear min-recall=3; **Cited needs 1**. Prefer scratch-on for operator agents.
- OpenClaw lean models may call search with weird args; verify living attach actually reaches CT101 MCP (`scripts/pi-gzmo-mcp-serve.sh` / `install-openclaw-living-attach.sh`), not a lab vault.
- Historical scar: first rsync missed `felt_use.rs` — after deploy always `rg FeltUseKind /opt/gzmo/current/gzmo-core` or run a binary that embeds Felt Use.

### 1.3 Instrumentation (ship if missing)

| Deliverable | Where | Notes |
|-------------|-------|-------|
| Per-search Felt Use counters | MCP status or `data/felt-use/session-*.json` | counts glance/cited touches per session |
| `gzmo_memory_status` field | `MemoryStatusReport` | optional: `felt_use_touches_session` |
| OpenClaw attach check | extend `scripts/living-mcp-attach-check.sh` | assert search increments CT101 honeypot recall for a probe id |

**Sidenote:** Do **not** add auto-search gym loops in overnight cron. Doctrine + `felt-use-ripen-floor.md`: side-effect of real work only. Crystallize campaign (section 2) is the deliberate exception and must be labeled as operator CORE pin, not gym.

### 1.4 OpenClaw / external agent wiring

**Files:** `scripts/openclaw-takeaway.sh`, `scripts/install-openclaw-living-attach.sh`, OpenClaw `mcp.servers.gzmo-living`.

**Subpoints**

1. Confirm Telegram/OpenClaw turns call `gzmo_memory_search` before answering ecosystem questions (AGENTS.ecosystem.md already points here).
2. Ensure `write_scratch` stays default true in tool schema descriptions so lean models don’t pass false.
3. After search, durable insights still go through **takeaway enqueue**, not Qdrant upsert.

**Sidenote:** Search alone does not crystallize IP into CORE; it only earns recall on *existing* honeypot facts. New insight text still needs section 2.

### 1.5 Acceptance

- [ ] `felt-use-depth.sh` shows rising `recall_ge1` after normal operator sessions (not only crystallize).
- [ ] `ripen/latest.json` advice leaves `starved_recall` after a week of real use **or** after crystallize (section 2).
- [ ] keep-quality `felt-use` row PASS when nonzero recall ≥ floor (`keep-quality-gate.sh`).

### 1.6 Out of scope

- Lowering `--min-recall`.
- Touching every vault row on ingest.
- Workstation overnight writer.

---

## 2. CORE crystallize campaign (intentional honey)

### 2.1 Goal

Turn ~15 insight-grade claims (product IP, not ADRs alone) into honeypot rows with origin `session_distill` (or `ingest`), confidence ≥0.90, then earn recall ≥3, then appear in fact export.

### 2.2 Candidate CORE set (starting list — edit with Max)

| # | Claim (one sentence) | Why insight-grade |
|---|----------------------|-------------------|
| 1 | GZMO is a Knowledge OS for one machine: LLMs generate, deterministic code verifies. | Product positioning |
| 2 | Cascading honeypot: sprawling web → distill → executable wisdom; forgetting clears intermediate pots. | Philosophy → mechanism |
| 3 | One living overnight writer: CT101 `gzmo-daemon` (ADR-0003). | Constitution |
| 4 | Disposable-vault: protect writer+recipe; row mass is not P0. | Doctrine |
| 5 | Nutrient path for external agents = takeaway enqueue / living attach, never raw Qdrant upsert. | Attach contract |
| 6 | Pipeline: extract → verify → promote → vault → honeypot → ripen → knowledge_core. | Cascade |
| 7 | Ripen dual gate: conf≥0.90 ∧ recall≥3 ∧ origin∈{ingest,verified_dream,session_distill}. | Honest Brain Feed |
| 8 | Felt Use grades: Glance/Cited/Bonded drive recall; promote starts at 0. | Metabolism |
| 9 | Lorenz/chaos modulates affect (temp/valence), not overnight writer authority. | Chaos honesty |
| 10 | Spark = serendipity; rare KG growth is expected, not failure. | Metric honesty |
| 11 | SessionDistill + TinyFolder/Herdr feed CT101 metabolism without CLI chat. | Nutrient |
| 12 | DualStackTrap: don’t trust idle WS serve LEDs; check CT101 daemon + vault counts. | Ops scar |
| 13 | Brain Feed GREEN after human Serendipity apply is a real gate. | USP LED |
| 14 | L.I.N.C. / neurosymbolic gates belong *before* honeypot promote (generator≠truth). | Architecture IP |
| 15 | Qdrant mirrors honeypot for search; not the write nutrient path. | Anti dual-brain |

**Sidenote:** Prefer short `[SYSTEM:…]` / `[DECISION:…]` / `[CONCEPT:…]` tagged one-liners that match existing honeypot style — easier FTS + profile.

### 2.3 New artifact (recommended)

Create `config/core-crystallize/CORE_CLAIMS.md` (or `.toml`) — human-owned list. Script reads it; does not invent claims.

### 2.4 Pipeline per claim

```text
claim text
  → scripts/openclaw-takeaway.sh / herdr-living-enqueue.sh  (NO --now)
  → CT101 distill worker / session close queue
  → SessionDistill extract+verify → promote_truths_with_origin(..., "session_distill")
  → honeypot row (recall=0) IF qualifies + derived bar
  → deliberate gzmo_memory_search (scratch on) targeting that content  → Cited (+3)
  → optional second search or Bonded pin (section 3)
  → export-knowledge-core.py includes row
```

**Code anchors**

```241:241:gzmo-core/src/session_distill.rs
            .promote_truths_with_origin(&truths, "session_distill")
```

```12:35:gzmo-core/src/memory/honeypot.rs
// qualifies_for_honeypot: conf≥0.85, source_file required, reject chat_*/sources/boilerplate
```

```186:204:gzmo-core/src/memory/lifecycle.rs
// session_distill is "derived": needs evidence OR confidence ≥ 0.92
```

**Critical sidenote — derived bar:** SessionDistill facts with conf in `[0.85, 0.92)` and **no evidence** are **vault-only** (never honeypot). Crystallize claims must ship with:

- `confidence ≥ 0.92`, **or**
- non-empty evidence_text from verify,

else they never enter the ripen candidate set.

**source_file sidenote:** Session distill uses `sessions/{id}.md` style paths to bypass `chat_session` excludes. Takeaway ritual must preserve that helper (`session_distill_source`). If you invent a manual SQL insert, set `source_file` to something that is **not** matching excluded substrings (`sources` is banned — don’t put claims under a path containing `sources`).

### 2.5 Script sketch: `scripts/core-crystallize.sh`

**Behavior**

1. Refuse if `gzmo-serve` active on WS.
2. Read CORE claims file.
3. For each claim: enqueue takeaway with stable prefix `CoreCrystallize: …`.
4. Optionally wait / trigger living distill **without** `--now` dual-writer (follow `ct101-takeaway-recall.sh` patterns).
5. After honeypot presence check (sqlite on CT101), run MCP/search reinforce queries (Cited).
6. Write artifact `data-next/core-crystallize/latest.{json,md}` with per-claim: enqueued, honeypot_id, recall_before/after, export_eligible.

**Thoughts**

- Batch ≤5 per day to avoid SessionDistill spam and origin soup.
- Idempotent: content_norm dedupe / lifecycle Extends should not create 15 duplicates of ADR-0003.
- Dry-run default; `CORE_CRYSTALLIZE_APPLY=1` to enqueue.

### 2.6 Acceptance

- [x] ≥10 CORE claims present as `is_latest=1` honeypot with allowed origin.
- [x] ≥10 with `recall_count≥3` and `confidence≥0.90`.
- [x] Dry-run export SQL lists them (15 CoreCrystallize dual+origin on CT101 2026-07-24).
- [ ] Operator profile / search returns them for “Knowledge OS” / “cascading honeypot” queries (needs living binary + embeddings; ensure_land leaves embedding NULL until search/embed backfill).

---

## 3. Seed Bonded on promote for doctrine / CORE tags (narrow)

### 3.1 Goal

Avoid waiting for accidental search for a **small allowlist** of CORE facts: on successful honeypot insert/upsert for matching rows, apply one `FeltUseKind::Bonded` (+5) so dual gate clears immediately.

### 3.2 Doctrine constraints

- **Allowlist only** — never Bonded-on-promote for all ingest (would make ripen meaningless).
- Prefer tag match: content starts with `CoreCrystallize:` or `container_tag` / decay_class reserved for CORE, or config list of content_norm hashes.
- Disposable-vault still applies: Bonded seeds CORE, not mass warehouse.

### 3.3 Implementation sketch

**File:** `gzmo-core/src/memory/vault.rs` after honeypot insert in `promote_truths_with_origin` (and lifecycle insert paths).

Pseudo:

```rust
// AFTER successful honeypot write for vault_id
if core_pin::should_seed_bonded(truth, origin) {
    let _ = felt_use::touch(self, vault_uuid, FeltUseKind::Bonded);
}
```

**New module:** `gzmo-core/src/memory/core_pin.rs`

- `should_seed_bonded(truth, origin) -> bool`
- Origins allowed: `session_distill`, `ingest` only (not raw `spark` unless verified).
- Match: prefix `CoreCrystallize:` OR config `[core_pin] prefixes = [...]` in `gzmo.toml`.

**Config (living toml — human pin):**

```toml
[core_pin]
enabled = true
bonded_on_promote = true
prefixes = ["CoreCrystallize:", "[CORE]"]
# optional: max bonded seeds per night
max_seeds_per_day = 20
```

**Sidenotes**

- Upsert path: ON CONFLICT does **not** reset `recall_count` (good). Don’t re-Bonded every upsert or recall explodes — seed only when `recall_count == 0` after insert, or only on first promote.
- Spark already Bonded-touches anchors after promote (`spark.rs` ~281–285). Don’t double-count confusion in metrics; CORE pin is for distill/ingest crystallize, not spark.
- Tests: unit test insert → recall_count == 5 for prefixed claim; non-prefixed stays 0.

### 3.4 Acceptance

- [ ] Prefixed crystallize claims reach export eligibility without 3 manual searches.
- [ ] Unprefixed ingest mass still starts at recall 0.
- [ ] `felt-use-depth` share_ge3 among felt rises without vault-wide gym.

### 3.5 Rollback

`[core_pin] bonded_on_promote = false` — no schema migration needed.

---

## 4. Origin trap cleanup

### 4.1 Goal

Facts that pass dual conf+recall but sit on origin `spark` / `manual` / `verified_spark` / bare `dream` must not silently miss CORE export (`origin_filter` advice).

### 4.2 Current export filter

```54:54:scripts/export-knowledge-core.py
ORIGINS = ("ingest", "verified_dream", "session_distill")
```

Mirrored in `SqliteVault::ripen_gate_census` SQL.

### 4.3 Options (pick one in ADR; don’t do all)

| Option | Change | Pros | Cons |
|--------|--------|------|------|
| **A. Crystallize → session_distill only** | Campaign + takeaway path only | No export change; doctrine-aligned | Spark serendipity still origin-filtered |
| **B. Add `operator_core` origin** | New origin string + export allowlist + promote API | Explicit CORE pin | Touches many match arms |
| **C. Allow `verified_spark` when Bonded+evidence** | Expand ORIGINS carefully | Serendipity can ripen | Dilutes gate story |

**Recommendation:** **A for campaign now**; **B** if Max wants a permanent pin API; avoid C until Brain Feed story is stable.

### 4.4 If implementing B (`operator_core`)

**Touch list**

1. `export-knowledge-core.py` `ORIGINS`
2. `vault.rs` `ripen_gate_census` SQL
3. `ripen_cmd.rs` help text
4. `lifecycle.rs` — decide if `operator_core` is derived (probably **not** — treat like ingest)
5. `honeypot-lifecycle-check.sh` origin lists
6. Docs: `docs/CT101_DEPLOY.md`, `MACHINE.md`, `BRAIN_FEED.md`
7. CLI: `gzmo memory pin-core "..."` → promote with origin `operator_core` + Bonded seed

**Sidenote:** Profile SQL in `profile.rs` already allows broader origins for ranking than export — profile ≠ CORE. Don’t confuse them in handoff PRs.

### 4.5 Acceptance

- [ ] `ripen/latest.json` no longer shows `origin_filter` for intentionally pinned CORE rows.
- [ ] Diagnostics distinguish `starved_recall` vs `origin_filter` in overnight narrative (already in export script advice).

---

## 5. Separate library from honey (purposeful forgetting)

### 5.1 Goal

Stop research corpus / takeout bleed from competing with CORE in honeypot profile and ripen attention. Cascade requires clearing intermediate pots.

### 5.2 Ingress hygiene (prevent)

**Already in `qualifies_for_honeypot`:**

- Reject `source_file` containing `sources`, `chat_history`, `chat_session`, `quelltext`
- Reject boilerplate (“sources do not contain”, “takeout drive”, …)

**Extend thoughtfully**

- Add reject substrings for known takeout paths if still leaking (`notebooklm`, `drive_clean`, `takeout`) — **test against false positives** on legitimate `source_file` names.
- Ingest-quality scripts: mark research dir as `library_only` (Pi Qdrant) without honeypot promote.

**Sidenote:** Pi knowledge collection is the right home for sprawling research. Honeypot is curated drops. Don’t “fix CORE” by promoting more library.

### 5.3 Residue cleanup (forget)

| Tool | Location | Status |
|------|----------|--------|
| forget-lint sibling | `/home/gzmo/github-clone/forget-lint` (if present) | plan/apply outside core |
| `gzmo_memory_forget` MCP | Spec’d, **unimplemented** in living MCP | opportunity |
| Manual SQL supersede | `is_latest=0` | last resort; prefer lifecycle |

**Handoff for forget MCP (future slice)**

1. Spec in MEMORY docs: soft forget = supersede + decay, hard = tombstone.
2. Gate: operator-only; never overnight auto-mass-delete without dry-run artifact.
3. Prefer targeting boilerplate / duplicate ADR clones / empty takeaway stamps.

### 5.4 Profile / search ranking

`profile.rs` orders by `recall_count DESC, confidence DESC` — once Felt Use works, CORE with recall outranks virgin library tags. Forgetting accelerates that; Felt Use alone slowly buries noise.

### 5.5 Acceptance

- [ ] Operator profile preferences no longer dominated by Word/PRISMA/amd-pstate trivia (spot check).
- [ ] Search for “cascading honeypot” returns crystallize claims above takeout titles.
- [ ] No mass honeypot delete without dry-run artifact + Max approve.

---

## 6. Path honesty (fact export vs charter cards vs Rust ripen)

### 6.1 Goal

One living truth for overnight CORE emit; docs and jobs agree.

### 6.2 Living truth (today)

| Path | Schema | Overnight? |
|------|--------|------------|
| `scripts/export-knowledge-core.py` | `knowledge_core` fact rows | **Yes** — `honeypot_ripen` job |
| `scripts/ripen-knowledge-core.py` | concept cards | No (charter / lab) |
| `memory/ripen.rs` `ripen_honeypot` | concept cards API | Compiled; not CT101 emit |

### 6.3 Tasks

1. Audit CT101 `gzmo.toml` `[orchestration.jobs.honeypot_ripen]` prompt vs `gzmo.toml.example` (~384–386).
2. Add banner comment at top of `ripen-knowledge-core.py` and `ripen.rs`: “NOT living overnight emit.”
3. `docs/CT101_DEPLOY.md` already notes starved_recall — link this handoff.
4. Dream/overnight agent prompt: print `ripen/latest.json` advice verbatim (don’t invent “core empty”).

### 6.4 Acceptance

- [ ] `gzmo ripen status` on CT101 matches export script census.
- [ ] No doc claims living uses concept-card ripen without labeling it alternate.

---

## 7. Suggested ship order (PRs)

| PR | Slice | Risk | Depends |
|----|-------|------|---------|
| 1 | §1 prove + attach instrumentation + docs pointer | Low | — |
| 2 | §2 `CORE_CLAIMS` + `core-crystallize.sh` dry-run/apply | Low–med | living enqueue |
| 3 | §3 `[core_pin]` Bonded-on-promote allowlist | Med | §2 prefixes |
| 4 | §4 origin ADR (A now; B optional pin CLI) | Med | product call |
| 5 | §5 forget / ingress reject extensions | Med–high | Max approve |
| 6 | §6 doc/job honesty cleanup | Low | — |

**Do not combine** Bonded-on-promote + origin expansion + forget in one PR.

---

## 8. Verification cheat sheet (CT101)

```bash
# Census
bash scripts/felt-use-depth.sh
ssh ct101 'GZMO_CONFIG=/opt/gzmo/gzmo.toml /opt/gzmo/current/target/release/gzmo ripen status'

# Export dry (same gates as overnight)
ssh ct101 'python3 /opt/gzmo/current/scripts/export-knowledge-core.py \
  --db /opt/gzmo/data/vault.db --output /opt/gzmo/data/knowledge_core.db \
  --min-confidence 0.90 --min-recall 3'
ssh ct101 'cat /opt/gzmo/data/ripen/latest.json'

# Dual-writer refuse
systemctl --user is-active gzmo-serve.service   # must not be active overnight

# Nutrient write
bash scripts/openclaw-takeaway.sh 'CoreCrystallize: …'
```

---

## 9. Explicit non-goals

- Memory-gym cron that searches random queries to inflate recall.
- Lowering min-recall / min-confidence to paint Brain Feed green.
- Second overnight writer on workstation.
- Qdrant upsert as OpenClaw nutrient path.
- Treating Pi knowledge chunk titles as CORE without crystallize.

---

## 10. Open questions for Max (product forks)

1. Finalize the 15 CORE claims list (edit section 2.2).
2. Prefer origin strategy **A** (session_distill only) vs **B** (`operator_core` pin CLI)?
3. Allow Bonded-on-promote (§3) on living CT101, or lab-next first?
4. Forget-lint: soft supersede only, or hard delete for takeout residue?

---

## 11. Pointer index (read before coding)

| Topic | Path |
|-------|------|
| Ripen honesty scar | `research/ripen-honesty-2026-07-20.md` |
| Felt Use floor | `research/opportunities/felt-use-ripen-floor.md` |
| Felt Use mass (parked) | `research/opportunities/felt-use-mass-growth.md` |
| Deploy / starved note | `docs/CT101_DEPLOY.md` |
| Cascade one-liner | `MACHINE.md` |
| Export gates | `scripts/export-knowledge-core.py` |
| Felt Use API | `gzmo-core/src/memory/felt_use.rs` |
| Search touch | `gzmo-core/src/platform_memory.rs` |
| Qualify | `gzmo-core/src/memory/honeypot.rs` |
| Derived bar | `gzmo-core/src/memory/lifecycle.rs` |
| Spark Bonded | `gzmo-core/src/spark.rs` |
| Takeaway wrapper | `scripts/openclaw-takeaway.sh` |
| Serendipity → takeaway | `scripts/serendipity-promote.sh` |
| Toml pin (not CORE DB) | `scripts/brain-intel-promote.sh` |

---

*End of handoff. Implement in the PR order in §7 unless Max reorders.*
