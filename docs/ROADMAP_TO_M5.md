# Roadmap to M5 — local system first

**Status:** 2026-06-04  
**Identity:** [`MACHINE.md`](../MACHINE.md)  
**Milestone detail:** [`CEILING_ROADMAP.md`](./CEILING_ROADMAP.md)  
**Strategy:** Finish **local production-ready** (M3 + M4 + stable ops). Integrate OSS patterns (Mem0, Graphiti, …) **after** that — one module at a time.

---

## North star

**M5 — Mature DB:** Honeypot ripens into exportable `knowledge_core` (~≤10% of honeypot rows) — “our knowledge,” separate from vault soup and raw archives.

Until then: **operate the distillation pipeline** (verify → promote → honeypot).

---

## Progress (2026-06-04)

```text
M0 ██████████  Foundation + eval harness
M1 ██████████  Wave-1 baseline (baseline-m4-post-sprint)
M2 ██████████  Honeypot + Qdrant honeypot (682, 0% drift)
M3 ███████░░░  Cognition reads honeypot; dream/spark need real nightly signal
M4 ██░░░░░░░░  Eval tiers live; Recall@5 not blocking yet
M5 ░░░░░░░░░░  Mature DB
```

| Store | Count |
|-------|-------|
| Vault | ~2815 |
| Honeypot | 682 |
| Qdrant `honeypot` | 682 |
| Qdrant `knowledge` | 3245 (legacy, read-only) |

---

## Definition of done — “local system finished”

Not M5 yet. **Production-ready local** means:

| # | Criterion | Check |
|---|-----------|-------|
| 1 | Stack reliable | `./scripts/verify-production.sh` → exit 0 after reboot |
| 2 | Nightly jobs | dream / distill / spark / qdrant in `logs/daemon.log` |
| 3 | Recall = honeypot | `retrieval-probes.py` 3/3; 5 real questions feel relevant |
| 4 | Cognition | Dream **or** Spark produces non-trivial output (not only ops-skip) |
| 5 | Ingest gated | Wave 2/3 blocked; `[ingest]` only with eval gate |
| 6 | Eval habit | `eval-quick.sh` after pipeline changes |
| 7 | Legacy | Delete Qdrant `knowledge` only per [`M2_HONEYPOT_REPORT.md`](./M2_HONEYPOT_REPORT.md) checklist |

---

## Block A — Operations (ongoing)

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
./scripts/start-production.sh --daemon   # after reboot
./scripts/verify-production.sh
./scripts/memory-status.sh
./target/release/gzmo health
```

**Weekly:** memory counts; skim `logs/daemon.log`.  
**Do not:** mass Takeout ingest, FrankenMoE `:8010`, new infra layers.

---

## Block B — Finish M3 (priority)

**Goal:** Dream and Spark reason over the **honeypot field**, not episodic janitor soup.

| ID | Task | Exit |
|----|------|------|
| B1 | Episodic filter / real work in `memory/YYYY-MM-DD.md` (not only daemon meta) | Dream runs extract (not `filtered < min`) |
| B2 | Tune `[spark]` pools (`anchor_min_age_hours`, decay classes) | Spark pairs anchor+recent or clear skip reason |
| B3 | Dream: Qdrant top-k honeypot + episodic as prose only | No `[ingest]` noise in `DREAMS.md` |
| B4 | Unified recall entry (honeypot FTS + vector + optional graph) | Agents default to crystal |

```bash
cargo test -p gzmo-core spark_recent_pool_reads
./target/release/gzmo spark
./target/release/gzmo dream
```

**M3 done when:** one week with meaningful nightly output + 3 manual recall questions answered from honeypot.

---

## Block C — M4 “good enough” (parallel)

| ID | Task | Exit |
|----|------|------|
| C1 | Golden 15 → 50 ([`M4_CONTINUOUS_EVAL_PLAN.md`](./M4_CONTINUOUS_EVAL_PLAN.md)) | Expanded manifest |
| C2 | Recall@5 in `report.json` ([`M4_MEMSCORE_RECALL5.md`](./M4_MEMSCORE_RECALL5.md)) | Non-null metric |
| C3 | Pre-ingest quarantine hook | No silent pollution |
| C4 | Habit: Tier 0 default, Tier 3 only for baseline | Documented in PR/commit |

```bash
scripts/ingest-quality/eval-quick.sh
scripts/ingest-quality/replay-wave-core.sh   # after prompt/ingest changes
```

**M4 done when:** Recall@5 ≥ 85%, faithfulness ≥ 0.9, anti-entities 0 — safe to enable one controlled ingest wave.

---

## Block D — Controlled feed (after B + C)

| Step | Action |
|------|--------|
| D1 | Sessions → `data/sessions/` (distill 02:15 UTC) |
| D2 | Small drops in `~/Schreibtisch/knowledge/` |
| D3 | Wave 2 from `sidecar-migration` only after M4 gate + operator OK |
| D4 | Never bulk-ingest full Takeout without replay |

---

## M5 — Mature DB (after local finished + months of collect)

| Phase | Action | Tool |
|-------|--------|------|
| 5.1 Collect | 3+ months honeypot growth from ingest, session_distill, verified_dream | — |
| 5.2 Review | Weekly human/golden approve for core candidates | `ripen-knowledge-core.py` (manifest) + `--approve` |
| 5.3 Ripen | Global dedup, contradiction, concept cards | `ripen-knowledge-core.py` |
| 5.4 Export | `knowledge_core.db` + manifest | `ripen-knowledge-core.py --commit` |
| 5.5 Index | Qdrant `knowledge_core` collection | `sync-knowledge-core-to-qdrant.py` (or `--sync-qdrant` on ripen commit) |
| 5.6 Profile | `profile.static` primarily from core | `profile.rs` reads `data/knowledge_core.db` when present |

**M5 exit:** exportable core; wave 2+ feeds core only when approved; ≤10% of honeypot row count.

**Scaffold (2026-06-07):** ripen/export pipeline implemented — `scripts/ripen-knowledge-core.py`
(+ `.sh` weekly wrapper). Defaults enforce the charter gate (residency ≥30d, corroboration ≥3),
so a strict run today yields **0 cards** (honeypot is days old — expected pre-M5). Validate the
pipeline now with relaxed gates:

```bash
# preview a v0 candidate core (writes data/knowledge_core.candidates.json only)
scripts/ripen-knowledge-core.py --min-age-days 0 --min-corroboration 1
# commit after operator review → data/knowledge_core.db + knowledge_core_export.md
scripts/ripen-knowledge-core.py --min-age-days 0 --min-corroboration 1 --commit
```

The ≤10% compaction exit gate is enforced by a hard cap; concept cards are importance-ranked
(distinct-fact density · confidence · corroboration · recall).

See [`MEMORY_ARCHITECTURE_SPEC.md`](./MEMORY_ARCHITECTURE_SPEC.md) §2 (Core layer).

---

## Explicitly later (integration backlog)

| Piece | Borrow from | When |
|-------|-------------|------|
| Graph lifecycle `update`/`derives` | Supermemory / Graphiti | After M3 |
| Auto-capture hooks | agentmemory | After local stable |
| Temporal queries | Zep/Memento | Optional on Neo4j |
| Cloud second node | — | Not SoT |

**Rule:** Pipeline unchanged — raw → verify → honeypot. One external pattern per quarter max.

---

## This week (concrete)

| Day | Action |
|-----|--------|
| Done | `verify-production.sh` OK; probes 3/3; health OK (sovereign FAIL expected) |
| Next | **B1:** write real episodic content or ingest one doc — unblock dream |
| Next | **B2:** one `[spark]` config tweak; re-run `gzmo spark` |
| Avoid | Wave 2 ingest, new subsystems, OSS ports |

---

## Commands (copy-paste)

```bash
# Daily / after reboot
./scripts/verify-production.sh
./scripts/memory-status.sh

# After Rust or prompt change
scripts/ingest-quality/eval-quick.sh

# M3 check
./target/release/gzmo spark && ./target/release/gzmo dream

# Before re-enabling live ingest
scripts/ingest-quality/gate-wave1-before-ingest.sh
```

---

## Changelog

| Date | Change |
|------|--------|
| 2026-06-07 | M5 scaffold: ripen/export, Qdrant `knowledge_core` sync, profile.static from core, cognition-stack seed |
| 2026-06-04 | Initial roadmap — local-first path to M5 |
