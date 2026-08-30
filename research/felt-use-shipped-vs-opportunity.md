# Felt Use: shipped on main vs opportunity leftover

**Date:** 2026-08-30  
**Status:** Research only (no product code)  
**Ticket:** [#224](https://github.com/maximilianwruhs-cyber/GZMO/issues/224) · map [#221](https://github.com/maximilianwruhs-cyber/GZMO/issues/221)  
**Question:** Diff `research/opportunities/felt-use-mass-growth.md` and soaked `felt-use-ripen-floor.md` against `origin/main` (`felt_use.rs`, vault utility boost, `scripts/felt-use-depth.sh`). What is already true on this Keep (`~/.gzmo-living`), what is still missing for a spec, what is CT101-only leftover.  
**Tip:** `origin/main` @ `8fc07a2`. Opportunity files on this sitting are **byte-identical** to `origin/main` (empty `diff`).

---

## Verdict (short)

**Done-when 1 of `felt-use-mass-growth` is shipped.** This Keep already has `honeypot.utility_score` (schema v10), Q-select after RRF/rerank, Glance Q=0, and a local `felt-use-depth.sh` census. **Done-when 2–3 are not.** Depth is honest **HOLD** (recall≥3 = 79, floor 100). Brain Feed GREEN is not proven here. **Zero** honeypot/semantic_vault rows mention Felt Use / MemRL / `utility_score`, so product search cannot retrieve doctrine — that is the spec gap, not another mechanism PR.

The 2026-08-16 telescope line that “living mass remains CT101-only until `#166` / harvest-organs binaries are on `/opt/gzmo/current`” is **stale leftover**. `#166` / `#167` / `#193` are on `origin/main`. This host has **no** `/opt/gzmo`. The living binary is `~/.local/bin/gzmo` → repo `target/release/gzmo`.

---

## 1. Opportunity claims vs `origin/main`

### 1.1 `felt-use-mass-growth` (status: `active`)

| Done-when | Opportunity text | On `origin/main`? | On this Keep? |
|-----------|------------------|-------------------|---------------|
| 1 | Living vault has `honeypot.utility_score` (v8/v9) and MCP/search orders by it | **Yes** — v8 add + v9 repair in [`vault.rs`](../gzmo-core/src/memory/vault.rs); Q-select [`apply_utility_select`](../gzmo-core/src/memory/vault.rs) → [`apply_utility_boost`](../gzmo-core/src/memory/recall_rrf.rs) ([#166](https://github.com/maximilianwruhs-cyber/GZMO/pull/166), merged 2026-08-16) | **Yes** — `PRAGMA user_version=10`; column + `idx_honeypot_utility` present |
| 2 | Weekly `felt-use-depth.sh` + utility census show **rising** dual-gate / utility mass from real sessions | Script reports `utility_positive` / avg / max ([`felt-use-depth.sh`](../scripts/felt-use-depth.sh)). **No** weekly-rising gate in-tree | **Snapshot only.** HOLD, not rising. See §2 |
| 3 | Brain Feed stays GREEN; no memory-gym | Depth thin = HOLD not RED ([`BRAIN_FEED.md`](../docs/BRAIN_FEED.md)). Gate still SSHes CT101 when depth is thin ([`brain-feed-check.sh`](../scripts/brain-feed-check.sh)) | **Not GREEN.** Depth HOLD. No `data-next/brain-feed/` artifact. No gym run this sitting |

Telescope (2026-08-16, still in the opportunity file): Q-select, Glance no longer mints Q, Outcome Q from later takeaway cite, depth script reports utility. **Those four sentences match main.** The next sentence — mass remains CT101-only until `#166`/harvest-organs on `/opt/gzmo/current` — does **not**.

### 1.2 `felt-use-ripen-floor` (status: `soaked`)

| Done-when | Opportunity text | On `origin/main`? | On this Keep? |
|-----------|------------------|-------------------|---------------|
| 1 | Census artifact: latest / recall≥1 / recall≥3 (+ share) | **Yes** — [`felt-use-depth.sh`](../scripts/felt-use-depth.sh) (2026-07-20, [#103](https://github.com/maximilianwruhs-cyber/GZMO/pull/103) share-among-felt) | **Yes** — this sitting wrote `data-next/felt-use-depth/latest.json` (gitignored) |
| 2 | Brain Feed / keep-quality row documents the floor without lying when starved | **Yes** — thin → HOLD ([`KEEP_QUALITY.md`](../docs/KEEP_QUALITY.md)) | HOLD advice names the floor. Starved = false |
| 3 | Optional: ripen status delta when floor improves — no second overnight writer | `gzmo ripen status` wired in the census script | Ripen CLI works locally. `knowledge_core.db` and `data/ripen/latest.json` are **missing** |

CT101 baseline in the soaked file (2026-07-20): latest≈38743 / ge1≈107 / ge3≈60. **Not this Keep.**

---

## 2. Already true on this Keep (`~/.gzmo-living`)

Measured 2026-08-30 against `/home/gzmo/.gzmo-living/data/vault.db` (MCP `gzmo_memory_status`: `vault_facts=4711`, `honeypot_latest=3005`, scratch Redis, control plane owner). Census SQL is the same six aggregates as [`felt-use-depth.sh`](../scripts/felt-use-depth.sh).

| Fact | Value | Source |
|------|-------|--------|
| Schema | `user_version=10`; `honeypot.utility_score REAL NOT NULL DEFAULT 0.0`; `idx_honeypot_utility` | `PRAGMA user_version` / `PRAGMA table_info(honeypot)` / `sqlite_master` |
| Depth census | latest **3005** · recall≥1 **104** · recall≥3 **79** · share_ge3 (of felt) **0.759615** · share_ge3_of_latest **0.02629** | `felt-use-depth.sh` 2026-08-30T19:39:58Z |
| Utility census | utility_positive **78** · avg **0.211647** · max **155.0** | same |
| Floors | `min_ge3=100`, `min_share_ge3=0.40` → **HOLD** (`depth_ok=false`, `ok=true`) | script defaults + this sitting |
| Ripen | dual gate **79** · dual+origin **79** · nonzero recall **104** · not starved · `knowledge_core.db` missing · `ripen/latest.json` missing | `GZMO_CONFIG=~/.gzmo-living/gzmo.toml gzmo ripen status` |
| Origin mix | **3005 / 3005** latest rows are `origin=ingest` | `SELECT origin, COUNT(*) … GROUP BY origin` |
| Utility vs recall | 68 of 78 `utility_score>0` equal `recall_count` (v8 seed `CAST(recall_count AS REAL)` in [`vault.rs`](../gzmo-core/src/memory/vault.rs)). 10 diverge (utility **below** recall). 26 felt rows have recall>0 and utility=0 | same vault |
| Doctrine text | **0** latest honeypot rows and **0** `semantic_vault` rows match `%Felt Use%` / `%felt_use%` / `%felt-use%` / `%MemRL%` / `%utility_score%` | sqlite `LIKE` |
| MCP search | Query `Felt Use felt_use MemRL utility_score` (limit 8, no scratch write) returned Prometheus / psutil / Fast Mode / Quality / Nomic / llava / midi-metrics / Notion — lexical “Used …” collisions, not doctrine | `gzmo_memory_search` |
| Wiki | Disabled | `~/.gzmo-living/gzmo.toml` `[wiki] enabled = false`; `gzmo_wiki_search` refused |
| Retrieval sidecars | Qdrant `:6333` ready; embed `:8081` Qwen3-Embedding-0.6B-Q8_0; rerank `:8082` listening. Toml has `[embeddings]` / `[qdrant]` / `[rerank]` enabled | `ss` + `curl` + toml |
| Binary | `~/.local/bin/gzmo` → `/home/gzmo/Projects/GZMO/target/release/gzmo`. **No** `/opt/gzmo`, **no** `harvest-organs` on `PATH` | `ls` / `type` |
| Stale artifact | Older `data-next/felt-use-depth/latest.json` (16:42Z) was **RED** `felt_use_depth_unreachable` via `ssh ct101` NXDOMAIN. Current script on main prefers a local vault file ([`ce2bdca`](https://github.com/maximilianwruhs-cyber/GZMO/commit/ce2bdca) “census this Keep's vault without CT101 SSH”). Re-run this sitting: HOLD, no SSH | artifacts + `git log` |

Mechanism weights on main (unchanged vs this branch): Glance recall +1 / Q **0**; Cited +3 / +3; Bonded +5 / +5; Outcome +3 / +8 — [`felt_use.rs`](../gzmo-core/src/memory/felt_use.rs). MCP search touches Glance when scratch is off, Cited when scratch is written — [`platform_memory.rs`](../gzmo-core/src/platform_memory.rs). Outcome Q is `reinforce_outcome_from_new_truths` — [`vault.rs`](../gzmo-core/src/memory/vault.rs).

`apply_utility_boost` does **not** invent hits. It rescales in-pool Q with `UTILITY_POOL_LAMBDA=0.05` then re-sorts ([`recall_rrf.rs`](../gzmo-core/src/memory/recall_rrf.rs)). Graph-hint fallback orders `utility_score DESC` ([`vault.rs`](../gzmo-core/src/memory/vault.rs)). Without a doctrine row in the relevance pool, Q cannot surface Felt Use.

---

## 3. Still missing for a spec (map #221)

Map destination: how this Keep gets Felt Use **doctrine** into honeypot through a real ingest/promote path (no memory-gym), and how product search must prove the bet is findable (or fail honest-empty). Mechanism is already on main. These are the holes a spec still has to name:

| Hole | Why it is not done-when 1 | Owner ticket |
|------|---------------------------|--------------|
| Doctrine row | Zero honeypot / semantic_vault text. Search cannot retrieve a bet that was never ingested | #222 (how a note becomes a row) → #226 (which ingest path) |
| Search-success wording | This sitting’s MCP search returned collisions, not honest-empty. Spec must say ranked doctrine hit vs fail-closed empty vs both | #223 (why Glance/Prometheus) → #225 |
| Rising mass (done-when 2) | One HOLD snapshot ≠ weekly rising dual-gate / utility from **sessions**. All latest rows are `ingest`; top Q (155) is ChatGPT ingest seed, not Outcome | later map, or stay out of #221 per “not yet specified” |
| Brain Feed GREEN (done-when 3) | Depth HOLD. No Brain Feed artifact. Gate still has a CT101 SSH fallback when depth is thin | later map unless #221 explicitly takes the floor |
| Ripen export | Dual gate is ready (79) but `knowledge_core.db` / `ripen/latest.json` are missing — optional soaked done-when 3 | later map |
| `CONTEXT.md` glossary | Map notes no `CONTEXT.md` in repo. Confirmed: **none** on `origin/main` | charting leftover |
| Hybrid vs FTS | Embed + Qdrant + rerank are **up** on this box; doctrine still absent, so collisions are not “FTS-only because vectors are off.” Spec must not assume enabling embed fixes findability | #221 notes; #223 |

Out of scope for the spec (already decided on the map): Jules/empty-input PRs, theater glass, memory-gym census, `feat/living-research-intel` as product tip.

---

## 4. CT101-only leftover

| Leftover | Where | What to do with it |
|----------|-------|--------------------|
| “Mass remains CT101-only until `#166`/harvest-organs on `/opt/gzmo/current`” | [`felt-use-mass-growth.md`](opportunities/felt-use-mass-growth.md) Telescope 2026-08-16 | **Stale.** `#166` is on main. This Keep already runs the mechanism. Do not block the spec on `/opt/gzmo/current` |
| Default SSH host `ct101`, vault `/opt/gzmo/data/vault.db`, bin `/opt/gzmo/current/target/release/gzmo` | [`felt-use-depth.sh`](../scripts/felt-use-depth.sh) | Fallbacks only. Local vault file short-circuits SSH ([`ce2bdca`](https://github.com/maximilianwruhs-cyber/GZMO/commit/ce2bdca)). This host has neither `/opt/gzmo` nor a resolvable `ct101` |
| Thin-depth Brain Feed row re-queries via `ssh ct101` | [`brain-feed-check.sh`](../scripts/brain-feed-check.sh) | **Still CT101-shaped.** Local census already has `recall_ge1`. A product fix would reuse the local JSON; not this ticket |
| `KEEP_QUALITY_VAULT_DB` default `/opt/gzmo/data/vault.db` | [`brain-feed-check.sh`](../scripts/brain-feed-check.sh); [`KEEP_QUALITY.md`](../docs/KEEP_QUALITY.md) | Docs still say “CT101 reference” |
| CT101 baseline string `~38743 / 107 / 60` | soaked opportunity + depth JSON `baseline_note` | Historical. Do not treat as this Keep’s floor |
| “CT101 census attempted” as kickoff done-when | [`docs/templates/AGENTIC_MEMORY_HARNESS_RESEARCH_PROMPT.md`](../docs/templates/AGENTIC_MEMORY_HARNESS_RESEARCH_PROMPT.md) | Telescope-era. This Keep is the prove-first vault |
| “Next grafts wait on CT101 living census” | [`research/lineage-watch/README.md`](lineage-watch/README.md) | Census **is** runnable here. Mass is thin, not unreachable |
| `/opt/gzmo/current`, `harvest-organs` binary | opportunity telescope; host `PATH` | **Absent.** Not a ship blocker for the spec |
| `ct101-takeaway-recall` Brain Feed row | [`brain-feed-check.sh`](../scripts/brain-feed-check.sh) | Name leftover. Living proof is a different ticket |

`felt-use-ripen-floor` itself is correctly **soaked** (census + honest HOLD). What is leftover is the **CT101 baseline numbers** and scripts/docs that still SSH when the living vault is this laptop.

---

## 5. What a spec should treat as already decided

From map #221 + this sitting:

1. **Mechanism** = graded Felt Use / MemRL Q already in `gzmo-core` (`felt_use.rs`, `apply_utility_boost`, Glance Q=0, Outcome via later cite). Do not re-ship [#166](https://github.com/maximilianwruhs-cyber/GZMO/pull/166) / [#167](https://github.com/maximilianwruhs-cyber/GZMO/pull/167) / [#193](https://github.com/maximilianwruhs-cyber/GZMO/pull/193).
2. **Doctrine row** = honeypot fact(s) that *state* the bet so search can retrieve them. **Missing** on this Keep.
3. **Prove-first Keep** = `~/.gzmo-living/data/vault.db`. Product tip is `origin/main`. Any later code change lives in `gzmo-core`.
4. **No gym.** Census is side-effect only. This sitting did not open memory-gym chats to inflate recall.

Census numbers here are the same snapshot [#227](https://github.com/maximilianwruhs-cyber/GZMO/issues/227) asked for (latest / ge1 / ge3 / utility_positive / max). Reuse; do not gym a second measurement.

---

## Cite

- [`research/opportunities/felt-use-mass-growth.md`](opportunities/felt-use-mass-growth.md) (identical to `origin/main`)
- [`research/opportunities/felt-use-ripen-floor.md`](opportunities/felt-use-ripen-floor.md) (identical to `origin/main`)
- [`gzmo-core/src/memory/felt_use.rs`](../gzmo-core/src/memory/felt_use.rs)
- [`gzmo-core/src/memory/vault.rs`](../gzmo-core/src/memory/vault.rs) (v8/v9 `utility_score`; `reinforce_felt`; `apply_utility_select`; `reinforce_outcome_from_new_truths`)
- [`gzmo-core/src/memory/recall_rrf.rs`](../gzmo-core/src/memory/recall_rrf.rs) `apply_utility_boost`
- [`scripts/felt-use-depth.sh`](../scripts/felt-use-depth.sh)
- [`scripts/brain-feed-check.sh`](../scripts/brain-feed-check.sh)
- PRs [#166](https://github.com/maximilianwruhs-cyber/GZMO/pull/166), [#167](https://github.com/maximilianwruhs-cyber/GZMO/pull/167), [#193](https://github.com/maximilianwruhs-cyber/GZMO/pull/193); commit [`ce2bdca`](https://github.com/maximilianwruhs-cyber/GZMO/commit/ce2bdca)
- Living vault `~/.gzmo-living/data/vault.db`; MCP `gzmo_memory_status` / `gzmo_memory_search`; `gzmo ripen status`
- MemRL field paper named by the opportunity: https://arxiv.org/html/2601.03192v2 (not re-read this sitting; mechanism mapping already in [`research/lineage-watch/README.md`](lineage-watch/README.md))
