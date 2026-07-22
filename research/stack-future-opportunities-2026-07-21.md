# Stack future development opportunities (90-day)

**Date:** 2026-07-21  
**Scope:** GZMO / airgap-living stack (+ sibling clones under `~/github-clone/`)  
**Method:** Primary sources only — repo docs, ADRs, scripts, artifacts, opportunity bets, uniqueness canvas. No blog summaries. Every claim cites a path (+ section where useful).

**Doctrine lock (living):**
- One overnight writer per vault ([ADR-0005](../docs/ADR-0005-flywheel-over-frozen-topology.md) Layer A; [ADR-0004](../docs/ADR-0004-airgap-living-usp.md))
- Promote-by-loop after beat-gate + operator ack ([LTL ADR-0003](../../little-tools-lab/docs/adr/0003-promote-by-loop.md); [CONTINUOUS_UPGRADE.md](../docs/CONTINUOUS_UPGRADE.md) Ring 3)
- Arena / calibration suggest-by-default — no silent overnight toml clobber ([ADR-0005](../docs/ADR-0005-flywheel-over-frozen-topology.md) §Layer C.5; [BRAIN_FEED.md](../docs/BRAIN_FEED.md) Hard rules)

---

## Already DONE (do not re-propose)

| Item | Evidence |
|------|----------|
| P0 `spark_field` refractory soft-pick | [CONTINUOUS_UPGRADE.md](../docs/CONTINUOUS_UPGRADE.md) §Ring 4 priority queue |
| P0 honeypot `classify_truth_pair` + knowledge beat | same |
| P0 herdr living enqueue | same; [BRAIN_FEED.md](../docs/BRAIN_FEED.md) takeaway/herdr; bet `herdr-living-enqueue-proof` soaked |
| P0 beat-gate kit 5/5 fixture | `data-next/beat-gate/latest.json` `ok=true` pass=5 fail=0 (2026-07-21) |
| P1 Obolus `beats_with_mad` | CONTINUOUS_UPGRADE Ring 4 |
| P1 Thought Cabinet crystallize (lab) | CONTINUOUS_UPGRADE Ring 4 |
| Nutrient bet wave (scores 18–23) | [research/opportunities/README.md](opportunities/README.md) — all listed bets `soaked` except `local-intel-32gb-128k` (`horizon`) |
| Brain Feed GREEN | `data-next/brain-feed/latest.md` Verdict GREEN (2026-07-21) |

---

## Executive ranking

Rank axes (as requested): (1) Brain Feed / nutrient / airgap-living USP · (2) uniqueness craft · (3) beat-gate honesty · (4) low dual-writer / promote risk.

| Rank | Id | Thrust | Priority | Horizon | USP | Craft | Beat | Dual-writer risk |
|------|----|--------|----------|---------|-----|-------|------|------------------|
| 1 | O1 | Refill Sense→Bet log (post-soak starvation) | P0 | days | ★★★★★ | ★★ | ★★★ | none |
| 2 | O2 | Honest soak nights (≥18h spacing → `soak_ready_unpark_ok`) | P0 | days–2w | ★★★★★ | ★ | ★★★★ | none |
| 3 | O3 | Grow Felt Use mass (`share_ge3_of_latest`) via real MCP work | P0 | weeks | ★★★★★ | ★★★ | ★★★ | low (side-effect) |
| 4 | O4 | Beat-gate versioned baselines + non-null `gate_passed` | P0 | 1–2w | ★★★★ | ★★★ | ★★★★★ | none (fixture) |
| 5 | O5 | First promote-by-loop (narrow cognition/knowledge) | ✅ | apply landed · overnight soak | ★★★★★ | ★★★ | ★★★★★ | **mutex required** |
| 6 | O6 | Spark lineage operator surface | ✅ | soaked | ★★★★ | ★★★★★ | ★★★ | none |
| 7 | O7 | REM substrate into dream recipe (fixture→beat) | P1 | 2–4w | ★★★★★ | ★★★★ | ★★★★ | low (lab first) |
| 8 | O8 | Arena night boringly reliable (suggest-only) | P1 | weeks | ★★★★ | ★★★ | ★★ | low (outside daemon) |
| 9 | O9 | Calibration / Arena human pin ritual (≥1 accept, ≥1 reject) | P1 | weeks | ★★★★ | ★★★★ | ★★ | low (explicit promote) |
| 10 | O10 | Serendipity capped weekly apply → vault mass | ✅ | soaked | ★★★★★ | ★★★ | ★★ | low (human apply) |
| 11 | O11 | Honeypot lifecycle / ripen visibility (Experience F) | P1 | 3–6w | ★★★★★ | ★★★★ | ★★★ | low |
| 12 | O12 | Port spark refractory + lifecycle goldens into LTL | P1 | 3–6w | ★★★ | ★★★★★ | ★★★★★ | none |
| 13 | O13 | Living organ-trace habit + missed-run watchdog | P2 | weeks | ★★★ | ★★ | ★★★ | none |
| 14 | O14 | Evidence-locate / faithfulness CI floor (W7) | P2 | 4–8w | ★★★ | ★★★ | ★★★★ | none |
| 15 | O15 | Stale-sweetness craft deepen (A→S path) | P2 | monthly Ring 4 | ★★★★ | ★★★★★ | ★★★ | none |

---

## Opportunity dossiers

### O1 — Refill Sense→Bet log (post-soak starvation) — P0 · days

**Thrust:** After the nutrient bet wave soaked, write **new candidate bets** from Sense scars — do not re-ship soaked plumbing.

**Why rare / USP:** Opportunity discovery is the operator automation for *what to build next* without tourism ([OPPORTUNITY_DISCOVERY.md](../docs/OPPORTUNITY_DISCOVERY.md) §Sense v2). Stack currently has `active_count` risk and zero candidates once missions complete ([research/opportunities/README.md](opportunities/README.md); `data-next/opportunity-discovery/sense-latest.md` advice path).

**Evidence:**
- Sense mines scars: felt depth, serendipity apply staleness, soak spacing, STACK gaps — [OPPORTUNITY_DISCOVERY.md](../docs/OPPORTUNITY_DISCOVERY.md) §Sense v2; [scripts/opportunity-sense.sh](../scripts/opportunity-sense.sh)
- Latest sense (2026-07-21): scars `felt_use_depth_thin`, `soak_samples_too_close` — `data-next/opportunity-discovery/sense-latest.md`
- `next-mission.md` still points at soaked `overnight-metabolism-triad` — stale mission pointer

**Next ship slice (1 PR):** New candidate bet files for O2–O5 (YAML frontmatter + Done when) + re-run `opportunity-sense.sh` / `opportunity-rank.sh` until one `active` bet; clear stale mission card.

**Risk:** Theater if bets restate soaked ids. **Soak:** none. **Dual-writer:** none.

---

### O2 — Honest soak nights — P0 · days–2 weeks

**Thrust:** Earn `soak_ready_unpark_ok` with ≥3 GREEN samples spaced ≥18h — gates **theater Unpark**, not craft.

**Why rare / USP:** Keep-quality is the living USP bar ([KEEP_QUALITY.md](../docs/KEEP_QUALITY.md); [ADR-0004](../docs/ADR-0004-airgap-living-usp.md)). Same-hour GREEN streaks are explicitly dishonest.

**Evidence:**
- `bash scripts/keep-quality-soak.sh --summary` → `soak_spacing_hold` · `honest_nights=1` · `spacing_rejects=2` · `need_nights=3` (2026-07-21 run)
- Soak log three GREENS within minutes on 2026-07-20 — `data-next/keep-quality/soak-log.jsonl`
- Sense scar `soak_samples_too_close` — `data-next/opportunity-discovery/sense-latest.md`
- Doctrine: soak gates theater only ([ADR-0005](../docs/ADR-0005-flywheel-over-frozen-topology.md) §Layer C.4; [UNPARK_ROADMAP.md](../docs/UNPARK_ROADMAP.md) prerequisite)

**Next ship slice:** Operator ritual only (no code): one soak sample per calendar night for 3 nights; optional timer already soaked (`soak-night-honest-timer`). Document advice in bet log.

**Risk:** Temptation to expand Unpark theater before honest_nights=3. **Dual-writer:** none.

---

### O3 — Felt Use mass growth (`share_ge3_of_latest`) — P0 · weeks

**Thrust:** Raise **vault-wide** recall≥3 mass via living MCP search as side-effect of real work — so ripen dual-gate stays honest at scale.

**Why rare / USP:** Overnight honeypot → ripen is the identity sentence ([UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md) §2; [BRAIN_FEED.md](../docs/BRAIN_FEED.md) Felt Use depth). Cloud memory toys do not close this loop.

**Evidence:**
- Felt depth GREEN among *felt* facts: `recall≥3=144/193` share_ge3≈0.75 — `data-next/felt-use-depth/latest.json`
- But `share_ge3_of_latest≈0.0037` on ~38k latest honeypot — same artifact; ripen dual_gate=131 vs latest=38863
- Keep-quality earlier HOLD on thin depth vs latest denominator — `data-next/keep-quality/latest.md` (2026-07-20)
- CONTINUOUS_UPGRADE W2: “Raise Felt Use depth floor” — [CONTINUOUS_UPGRADE.md](../docs/CONTINUOUS_UPGRADE.md) §W2
- Soaked measurement bet: [felt-use-ripen-floor.md](opportunities/felt-use-ripen-floor.md) — **next is mass, not another meter**

**Next ship slice:** Bet + operator checklist: living MCP attached during real sessions; weekly `felt-use-depth.sh` delta on `share_ge3_of_latest` / dual_gate; optional soft floor bump in keep-quality **advice only** (no memory-gym).

**Risk:** Memory-gym tourism (explicitly forbidden — [BRAIN_FEED.md](../docs/BRAIN_FEED.md)). **Dual-writer:** none if CT101 holds claim.

---

### O4 — Beat-gate versioned baselines + `gate_passed` honesty — P0 · 1–2 weeks

**Thrust:** Make kit PASS mean “beats versioned incumbent baseline,” not “recipe exited 0.”

**Why rare / USP:** Promote science for organ handoff ([STACK_OPPORTUNITY_MAP.md](../docs/STACK_OPPORTUNITY_MAP.md) o2; [CONTINUOUS_UPGRADE.md](../docs/CONTINUOUS_UPGRADE.md) Ring 3 / W3).

**Evidence:**
- Kit summary PASS 5/5 — `data-next/beat-gate/latest.md`
- But `gate_passed` is `true` only for **config**; cognition/knowledge/discovery/ops are `null` — `data-next/beat-gate/latest.json`
- Cognition meta has `beats_incumbent: true` with inline baseline mins, not a versioned committed baseline file — `data-next/beat-gate/metas/cognition.json`
- W3 explicitly: “Cognition + knowledge beat-gates fixture GREEN with **versioned baselines**” — [CONTINUOUS_UPGRADE.md](../docs/CONTINUOUS_UPGRADE.md) §W3
- Kit note: “Open eval kit spike — no CT101 writes” — `latest.json` `note`

**Next ship slice:** Check in baseline JSON per loop under `little-tools-lab` or `data-next/beat-gate/baselines/`; make `beat-gate-kit.sh` require non-null `gate_passed` for cognition+knowledge; CI fixture run.

**Risk:** Raising bar may FAIL kit until baselines honest — good. **Dual-writer / promote:** none until O5.

---

### O5 — First promote-by-loop (narrow) — ✅ LIVING APPLY LANDED 2026-07-22 (overnight soak pending)

**Thrust:** Exercise ADR-0005 / LTL-0003 once: beat-gate PASS + mutex + operator ack → hand off **one** loop into current living host.

**Why rare / USP:** Flywheel over frozen topology is the process USP ([ADR-0005](../docs/ADR-0005-flywheel-over-frozen-topology.md); [CT101_BOUNDARY.md](../docs/CT101_BOUNDARY.md)).

**Evidence:**
- Record ritual soaked — [promote-by-loop-first.md](opportunities/promote-by-loop-first.md)
- Living apply active — [promote-loop-living-apply.md](opportunities/promote-loop-living-apply.md): `PROMOTE_APPLY=1` knowledge-only; CT101 pin `knowledge.v1` + `session-to-dream.sh` handoff; rollback under `/opt/gzmo/data/beat-gate/promotions/rollback`
- Post-apply Brain Feed GREEN + living probe OK (2026-07-22)
- Disposable-vault doctrine: protect writer+recipe; felt-use mass parked as candidate

**Soak remaining:** BF + living probe GREEN after one overnight, then soak the bet.

**Risk:** Dual-writer if mutex skipped; silent graft if ack skipped. Multi-loop / cutover still refused.

---

### O6 — Spark lineage operator surface — ✅ SOAKED 2026-07-22

**Thrust:** Last successful SparkReport visible on `gzmo status` / Observatory — Experience B felt.

**Why rare / USP:** Triangular `stale_sweetness` serendipity is signature ([UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md) §3; canvas A-tier “Stale-sweetness spark anchors”).

**Evidence:**
- [SIGNATURE_EXPERIENCES.md](../docs/SIGNATURE_EXPERIENCES.md) Experience B — elevate last spark card
- [UNIQUENESS_BUILD_PLAN.md](../docs/UNIQUENESS_BUILD_PLAN.md) W3
- PIECE_ELEVATION_MAP: spark-link **ELEVATE** — “Surface spark lineage in Observatory + gzmo status”
- Canvas: A-tier stale-sweetness — `~/.cursor/projects/empty-window/canvases/code-uniqueness-tier-list.canvas.tsx`

**Next ship slice:** Persist SparkReport under `data-next/` from cognition-smoke; one status/Observatory card reading it; fixture proof.

**Risk:** Observatory redesign festival (explicit stop — UNIQUENESS_BUILD_PLAN). **Dual-writer:** none.

---

### O7 — REM substrate into dream recipe — ✅ SOAKED 2026-07-22

**Thrust:** Wire `rem-substrate` into `session-to-dream.sh` when anchors exist (ADR-0002-safe; fixture first).

**Why rare / USP:** Dream depth without inline DreamEngine ([SIGNATURE_EXPERIENCES.md](../docs/SIGNATURE_EXPERIENCES.md) Experience A; canvas B-tier “Honeypot REM neighbor pack”).

**Evidence:**
- [UNIQUENESS_BUILD_PLAN.md](../docs/UNIQUENESS_BUILD_PLAN.md) W4
- PIECE_ELEVATION_MAP: rem-substrate **ELEVATE** — fuse into session-to-dream when honeypot_rem lab-backed
- CONTINUOUS_UPGRADE prefers nutrient/uniqueness craft over tourism

**Next ship slice:** Fixture dream emits REM markdown section; knowledge beat-gate still PASS; no scheduler pedagogy/chaos.

**Risk:** Catalog gap / unwired claim. Lab-first keeps promote risk low.

---

### O8 — Arena night boringly reliable — ✅ SOAKED 2026-07-22 (artifact check; force-run optional)

**Thrust:** Overnight Arena → `data-next/arena/latest.json` + champion suggestion on a schedule operators trust — still suggest-only.

**Why rare / USP:** Ground truth = wall meter + living memory ([STACK_OPPORTUNITY_MAP.md](../docs/STACK_OPPORTUNITY_MAP.md) e1; [OBOLUS_ARENA_BOUNDARY.md](../docs/OBOLUS_ARENA_BOUNDARY.md)).

**Evidence:**
- CONTINUOUS_UPGRADE W4: “Arena night → champion suggestion boringly reliable”
- Latest Arena burst is **2026-07-18** (stale vs 2026-07-21) — `data-next/arena/latest.json`
- `energy_source: estimate` — RAPL not yet honest joules (STACK map e1 note; OBOLUS_ARENA_BOUNDARY)
- Boundary: never wire into `gzmo-daemon` by default — [OBOLUS_ARENA_BOUNDARY.md](../docs/OBOLUS_ARENA_BOUNDARY.md)

**Next ship slice:** Timer/cron outside daemon job set → `arena-night.sh`; assert suggestion sibling toml; brain-intel row still `auto_apply=false`.

**Risk:** Auto-pin theater. Keep human promote only ([BRAIN_FEED.md](../docs/BRAIN_FEED.md) P1).

---

### O9 — Calibration / Arena human pin ritual — ✅ SOAKED 2026-07-22

**Thrust:** Real pins: ≥1 accepted + ≥1 rejected-with-reason via `promote-fused` / `brain-intel-promote.sh`.

**Why rare / USP:** Fuse→human promote is signature Experience C ([SIGNATURE_EXPERIENCES.md](../docs/SIGNATURE_EXPERIENCES.md); UNIQUENESS_THESIS §4).

**Evidence:**
- Brain intel GREEN suggest-ready — `data-next/brain-intel/latest.md`
- CONTINUOUS_UPGRADE W4: “Calibration promote-fused used for real pins”
- ADR-0005: fast-pin OK via explicit script; no silent overnight clobber

**Next ship slice:** One documented pin decision log under `data-next/brain-intel/` (accept/defer/reject); no daemon toml write from Arena.

**Risk:** Accidental living engine swap. Mitigate: `--diff` then `--apply` only.

---

### O10 — Serendipity capped weekly apply → vault mass — ✅ SOAKED 2026-07-22

**Thrust:** Keep cadence → USP filter → review ≤3 → human apply ≤3/ISO-week as the nutrient pump.

**Why rare / USP:** Deliberate serendipity ≠ similarity RAG ([STACK_OPPORTUNITY_MAP.md](../docs/STACK_OPPORTUNITY_MAP.md) o5; BRAIN_FEED serendipity). Horizon/local-intel theater never enters living takeaways.

**Evidence:**
- Cap + filter: `scripts/serendipity-promote.sh` (`SERENDIPITY_WEEKLY_CAP`, horizon regex) + `scripts/serendipity-weekly-check.sh`
- Apply 2026-07-22: week **2026-W30 applies=1/3**; filtered TurboQuant ×2; USP takeaways Zellij / pi-telegram / GZMO enqueued on CT101
- Weekly check OK; Brain Feed GREEN — [serendipity-weekly-apply.md](opportunities/serendipity-weekly-apply.md)

**Habit next:** Re-run weekly when spark candidates clear; stay ≤3 applies/week; feel-use mass remains active P0.

**Risk:** Auto-apply (forbidden). **Dual-writer:** refuse if serve active (existing checks).

---

### O11 — Honeypot lifecycle / ripen visibility — ✅ SOAKED 2026-07-22

**Thrust:** Make Experience F undeniable: vault/honeypot/pending counts + lifecycle stages on operator surface.

**Why rare / USP:** MACHINE identity = Honeypot + verify + promote ([UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md) §2; canvas A-tier lifecycle).

**Evidence:**
- SIGNATURE_EXPERIENCES Experience F — “Not fully felt yet — elevation target #1”
- UNIQUENESS_BUILD_PLAN W2 (partially advanced; surface still thin)
- Felt ripen snippet shows dual+origin ready — `data-next/felt-use-depth/latest.json` ripen.snippet
- PIECE_ELEVATION: honeypot-gate **ELEVATE**

**Next ship slice:** Status/Observatory split counts (vault vs honeypot origin vs pending); fixture cognition with lifecycle outcomes; no CT101 vault import.

**Risk:** Importing 60k vault to “look busy” (explicit non-claim — UNIQUENESS_THESIS).

---

### O12 — LTL goldens for spark refractory + lifecycle — ✅ SOAKED 2026-07-21 (excluded from O7–O15 *queue* because already done)

**Thrust:** Port/deepen spark refractory + honeypot lifecycle tests into LTL golden fixtures (Ring 3 honesty).

**Why rare / USP:** S-tier spark refractory + A-tier lifecycle are craft kernels ([code-uniqueness-tier-list.canvas.tsx](file:///home/gzmo/.cursor/projects/empty-window/canvases/code-uniqueness-tier-list.canvas.tsx); CONTINUOUS_UPGRADE W3).

**Evidence:**
- CONTINUOUS_UPGRADE W3: “Port or deepen honeypot lifecycle / spark refractory tests into LTL goldens”
- P0 kernels marked Done in Ring 4 queue — next is **fixture fidelity**, not re-implementation
- Beat-gate kit is reference assembly — `data-next/beat-gate/latest.json`

**Next ship slice:** Golden fixtures in piece repos + cognition/knowledge beat recipes; kit PASS with non-null gate_passed (feeds O4/O5).

**Risk:** None on living if fixture-only.

---

### O13 — Living organ-trace habit + missed-run watchdog — ✅ SOAKED 2026-07-22

**Thrust:** Prove which organs fired overnight; alert if distill/dream missed >26h.

**Why rare / USP:** Proof of overnight organ use is rare; watchdog is commodity but protects singular product ([STACK_OPPORTUNITY_MAP.md](../docs/STACK_OPPORTUNITY_MAP.md) o1, r5).

**Evidence:**
- Living organ-trace bet soaked — [living-organ-trace.md](opportunities/living-organ-trace.md); STACK o1
- Missed-run watchdog exists as soft-fail — STACK r5
- CONTINUOUS_UPGRADE Ring 1 quality bar still needs nightly honesty

**Next ship slice:** Weekly `--living` organ-trace in operator cadence; watchdog YELLOW surface only (never flip GREEN math).

**Risk:** Dashboard theater. Keep file artifacts, not a new web app.

---

### O14 — Evidence-locate / faithfulness floor — ✅ SOAKED 2026-07-22 (local fixture; not GH Actions)

**Thrust:** Attach evidence spans to promote/verify; faithfulness fixture green in CI.

**Why rare / USP:** Agent output CI still rare ([STACK_OPPORTUNITY_MAP.md](../docs/STACK_OPPORTUNITY_MAP.md) r4); advances grounded promote ([UNIQUENESS_BUILD_PLAN.md](../docs/UNIQUENESS_BUILD_PLAN.md) W7).

**Evidence:** UNIQUENESS_BUILD_PLAN W7; STACK r4 faithfulness-ci exists — deepen wiring into cognition beat.

**Next ship slice:** Fixture cognition attaches evidence; CI job offline green; optional beat-gate hook.

**Risk:** Scope creep into pedagogy CI. Keep off GREEN overnight gate.

---

### O15 — Stale-sweetness craft deepen — ✅ SOAKED 2026-07-22 (via O6; monthly Ring 4 deepen remains)

**Thrust:** One monthly uniqueness craft PR: deepen spark-link scoring / refractory interaction; re-score canvas.

**Why rare / USP:** Ring 4 doctrine — unique kernels first-class ([ADR-0005](../docs/ADR-0005-flywheel-over-frozen-topology.md) §C.6; CONTINUOUS_UPGRADE Ring 4).

**Evidence:** Canvas A-tier stale-sweetness craft 4; CONTINUOUS_UPGRADE “prefer upgrades that increase Brain Feed nutrient or beat-gate honesty.”

**Next ship slice:** Single scoring/test PR in spark-link; beat cognition fixture; update canvas craft score.

**Risk:** Portfolio tour (stop at 1–2 kernels/month — CONTINUOUS_UPGRADE Ring 4).

---

## Deferred / theater (explicit anti-goals)

| Defer | Why | Source |
|-------|-----|--------|
| HSP dialect MIDI / metabolism sonification as strengthen | Demable theater; CONTINUOUS_UPGRADE P1 “Keep demable theater” | [BRAIN_FEED.md](../docs/BRAIN_FEED.md) Out/; CONTINUOUS_UPGRADE Ring 4; UNPARK Wave 2 |
| Pantheon / discovery theater / Pi glass / €/night display as next strengthen | Demoted Unpark waves | [BRAIN_FEED.md](../docs/BRAIN_FEED.md); [STACK_OPPORTUNITY_MAP.md](../docs/STACK_OPPORTUNITY_MAP.md); [UNPARK_ROADMAP.md](../docs/UNPARK_ROADMAP.md) |
| IpW / Forge as living required path | After Arena pin is routine; suggest-only | BRAIN_FEED P1b; OBOLUS_ARENA_BOUNDARY; CONTINUOUS_UPGRADE W4 |
| Local strong 128k+ on 32GB VRAM | Horizon bet — world not ready | [local-intel-32gb-128k.md](opportunities/local-intel-32gb-128k.md); OPPORTUNITY_DISCOVERY hard rule 2 |
| Cognis / escape-loop / ZPD on GREEN overnight | Never-as-brain | BRAIN_FEED Out/; UNPARK_ROADMAP Never-as-brain; STACK map |
| Full-assembly cutover before any loop promote | Superseded theater | ADR-0005; LTL ADR-0003 |
| Auto-promote Arena champions | Forbidden | ADR-0005 Non-goals; BRAIN_FEED Hard rules |
| Second overnight writer / dual-serve “for convenience” | Layer A invariant | ADR-0005; ADR-0004; brain-feed-check dual-writer row |
| Expanding Unpark Wave 1–2 brand while soak_spacing_hold | Soak gates theater | KEEP_QUALITY Unpark gate; soak summary 2026-07-21 |
| Memory-gym Cursor sessions to inflate recall | Credit tourism | BRAIN_FEED takeaway/Felt Use; OPPORTUNITY_DISCOVERY hard rule 3 |
| CT101 vault import to fake density | Dilutes uniqueness | UNIQUENESS_THESIS non-claims; UNIQUENESS_BUILD_PLAN stop-doing |
| AOS CE / marketplace / portable-core rewrite | Wave 4 later | STACK map Later; UNPARK Wave 4 |
| Observatory redesign festival | 2–3 signature moments only | UNIQUENESS_BUILD_PLAN stop-doing |
| Re-shipping soaked nutrient plumbing as “new” P0 | Bet log already soaked | opportunities/README.md |

---

## 90-day sequencing recommendation

Aligned to [CONTINUOUS_UPGRADE.md](../docs/CONTINUOUS_UPGRADE.md) §Workstreams, updated for 2026-07-21 craft state:

```text
Days 0–14 (W1 stabilize + O1/O2/O4)
  O1  Refill candidate bets from Sense scars (one active)
  O2  Three honest soak nights (≥18h) → soak_ready_unpark_ok
  O4  Versioned beat-gate baselines; non-null gate_passed for cognition+knowledge
  Keep BF GREEN; weekly beat-gate-kit habit → data-next/beat-gate/

Days 14–42 (W2 nutrient + O3/O10)
  O3  Felt Use mass via real living MCP (track share_ge3_of_latest / dual_gate)
  O10 Weekly serendipity apply ≤3 (habit, not new scripts)
  herdr/tinyFolder stay side-effect-only (already GREEN — do not gym)

Days 21–56 (W3 lab honesty + O5/O12/O7)
  O12 LTL goldens for refractory + lifecycle
  O5  First promote-by-loop (knowledge or cognition) under living-host-mutex
  O7  REM into dream recipe (fixture → knowledge beat PASS)

Days 42–90 (W4 suggestions + craft + O6/O8/O9/O11)
  O8  Arena night schedule boring
  O9  ≥1 accept + ≥1 reject human pins
  O6  Spark lineage on status/Observatory
  O11 Honeypot/ripen visibility (Experience F)
  Ring 4: at most 1–2 S/A craft PRs (O15); re-score uniqueness canvas
  Defer IpW/Forge until O8+O9 routine
```

**Mutex note:** Any living prove/promote uses `scripts/living-host-mutex.sh claim|release` ([CT101_BOUNDARY.md](../docs/CT101_BOUNDARY.md); CONTINUOUS_UPGRADE cheat sheet). Workstation living is allowed only under claim ([ADR-0005](../docs/ADR-0005-flywheel-over-frozen-topology.md) Layer B).

---

## Source index

| Path | Role |
|------|------|
| [docs/CONTINUOUS_UPGRADE.md](../docs/CONTINUOUS_UPGRADE.md) | Four rings, 90-day workstreams, Ring 4 Done queue, anti-goals |
| [docs/ADR-0005-flywheel-over-frozen-topology.md](../docs/ADR-0005-flywheel-over-frozen-topology.md) | Flywheel > frozen topology; mutex; promote-by-loop; suggest-by-default |
| [docs/ADR-0004-airgap-living-usp.md](../docs/ADR-0004-airgap-living-usp.md) | Airgap living USP invariants |
| [docs/BRAIN_FEED.md](../docs/BRAIN_FEED.md) | Nutrient satellites; hard rules; demoted theater |
| [docs/STACK_OPPORTUNITY_MAP.md](../docs/STACK_OPPORTUNITY_MAP.md) | Atlas of stack opportunities + keep/unpark lanes |
| [docs/UNIQUENESS_THESIS.md](../docs/UNIQUENESS_THESIS.md) | Signature claims + non-claims |
| [docs/UNIQUENESS_BUILD_PLAN.md](../docs/UNIQUENESS_BUILD_PLAN.md) | W1–W10 elevation workstreams |
| [docs/SIGNATURE_EXPERIENCES.md](../docs/SIGNATURE_EXPERIENCES.md) | Experiences A–F (spark/dream/fuse/honeypot) |
| [docs/OPPORTUNITY_DISCOVERY.md](../docs/OPPORTUNITY_DISCOVERY.md) | Sense→Rank→Bet; Sense v2 scars |
| [docs/OBOLUS_ARENA_BOUNDARY.md](../docs/OBOLUS_ARENA_BOUNDARY.md) | Arena outside daemon; suggest-only |
| [docs/CT101_BOUNDARY.md](../docs/CT101_BOUNDARY.md) | Reference living host + mutex + promote-by-loop |
| [docs/KEEP_QUALITY.md](../docs/KEEP_QUALITY.md) | USP quality bar + soak Unpark gate |
| [docs/UNPARK_ROADMAP.md](../docs/UNPARK_ROADMAP.md) | Theater waves vs Brain Feed; never-as-brain |
| [research/opportunities/README.md](opportunities/README.md) | Bet log (all nutrient bets soaked; local-intel horizon) |
| [little-tools-lab/docs/adr/0003-promote-by-loop.md](../../little-tools-lab/docs/adr/0003-promote-by-loop.md) | Lab→living promote policy |
| [little-tools-lab/docs/PIECE_ELEVATION_MAP.md](../../little-tools-lab/docs/PIECE_ELEVATION_MAP.md) | Elevate spark-link / rem / honeypot / fuse |
| `~/.cursor/projects/empty-window/canvases/code-uniqueness-tier-list.canvas.tsx` | S/A/B/C uniqueness kernels (2026-07-21) |
| [scripts/opportunity-sense.sh](../scripts/opportunity-sense.sh) | Sense scars + STACK gaps |
| [scripts/brain-feed-check.sh](../scripts/brain-feed-check.sh) | Brain Feed gate implementation |
| [scripts/beat-gate-kit.sh](../scripts/beat-gate-kit.sh) | Weekly kit → `data-next/beat-gate/` |
| [scripts/living-host-mutex.sh](../scripts/living-host-mutex.sh) | Living host claim/release |
| `data-next/beat-gate/latest.{json,md}` | Craft state: 5/5 PASS; gate_passed sparse |
| `data-next/felt-use-depth/latest.{json,md}` | Felt GREEN; of_latest thin |
| `data-next/brain-feed/latest.md` | BF GREEN 2026-07-21 |
| `data-next/keep-quality/` + soak summary | GREEN bar; `soak_spacing_hold` |
| `data-next/opportunity-discovery/sense-latest.md` | Scars: felt depth + soak spacing |
| `data-next/arena/latest.json` | Arena suggestion (stale 2026-07-18) |
| `data-next/serendipity/cadence-latest.md` | Cadence OK; applies_logged=1 |
| `data-next/brain-intel/latest.md` | Suggest-ready; auto_apply=false |
| `organ-audit/` (sibling) / LTL `catalog/projects/organ-audit.md` | Autonomic ownership linter (Ring 4 mention) |

---

*End of note. Prefer nutrient / uniqueness craft / beat-gate honesty. Defer Unpark theater until soak is honest; never dual-write; never auto-pin Arena.*
