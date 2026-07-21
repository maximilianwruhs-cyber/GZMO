# Continuous Upgrade Process

**Status:** Active plan (2026-07-21)  
**USP:** [ADR-0004-airgap-living-usp.md](./ADR-0004-airgap-living-usp.md)  
**Active strengthen lane:** [BRAIN_FEED.md](./BRAIN_FEED.md)  
**Quality bar:** [KEEP_QUALITY.md](./KEEP_QUALITY.md)  
**Unpark sequencing:** [UNPARK_ROADMAP.md](./UNPARK_ROADMAP.md)  
**Lab parity:** [ct101-systems/120-two-stack-boundary/beat-gates.md](./ct101-systems/120-two-stack-boundary/beat-gates.md)  
**Craft backlog:** uniqueness tier list (Cursor canvas) · S/A kernels first

## Goal

Run a **repeatable flywheel** that upgrades nutrient density of the living vault and the craft of unique kernels — without dual-writers, without auto engine swaps, and without promoting theater into the overnight brain.

## North-star metrics

| Metric | Source | Cadence | Pass signal |
|--------|--------|---------|-------------|
| Keep-quality soak | `keep-quality-soak.sh --summary` | Continuous / nightly | `soak_ready_unpark_ok` |
| Brain Feed GREEN | `brain-feed-check.sh` | Daily | All P0 satellites honest |
| Beat-gate fixture | `beat-gate.sh --loop … --fixture` | CI + weekly kit | Pass vs baseline |
| Arena suggestion | `arena-night.sh` → `data-next/arena/` | Overnight | Suggestion only (human pin) |
| Uniqueness craft | Tier list re-score | Monthly | S/A kernels improved or promoted |

## Hard boundaries (never reverse)

1. **One overnight writer** ([ADR-0003](./ADR-0003-one-instance-metabolism.md)) — workstation never runs living metabolism while CT101 owns the vault.
2. **No auto engine swap** — Arena / Forge / IpW / calibration emit **suggestions**; human pin only.
3. **Beat-gate ≠ cutover** — green `beat-meta.json` does not authorize CT101 grafts; require explicit `CUTOVER_APPROVED=1` for any migration tooling. **GZMO-next re-promotion** only after next beats CT101 at **all scales** (full kit GREEN, fixture + live) — see [CT101_BOUNDARY.md](./CT101_BOUNDARY.md).
4. **Brain Feed is the only strengthen claim** — HSP / pantheon / Observatory stay demable theater.
5. **Cognis / escape-loop / ZPD / attractor brand** — lab only; never GREEN overnight gate.
6. **Prefer nutrient / Brain Feed / airgap-living USP** — do not burn sessions on ecosystem tourism.

---

## Flywheel (four rings)

```text
┌─────────────────────────────────────────────────────────────┐
│  Ring 4 — Portfolio craft (monthly)                         │
│  uniqueness tier list → deepen S/A kernels → re-score       │
└────────────────────────────▲────────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────────┐
│  Ring 3 — Lab → production ladder (weekly)                  │
│  LTL fixture mature → beat-gate → human promote / handoff   │
└────────────────────────────▲────────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────────┐
│  Ring 2 — Intelligence suggestions (overnight)              │
│  Arena / calibration → data-next suggestions → human pin    │
└────────────────────────────▲────────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────────┐
│  Ring 1 — Living nutrient (daily)                           │
│  takeaway · tinyFolder · Felt Use · serendipity · soak      │
└─────────────────────────────────────────────────────────────┘
```

### Ring 1 — Living nutrient (daily)

**Owner:** operator on real work (side-effect only — no memory-gym sessions).

| Action | Command / path | Notes |
|--------|----------------|-------|
| Close ritual | `gzmo session close --takeaway` / herdr close | No `--now` on workstation while CT101 lives |
| tinyFolder → living | inbox drop → enqueue toward living | Target living host, not `~/.gzmo` lite |
| Felt Use | real MCP search (recall≥3 floor) | Feeds ripen honesty |
| Serendipity | `serendipity-promote.sh` dry-run → review → apply | ≤3 takeaways |
| Quality bar | `brain-feed-check.sh` + soak | BF GREEN ≠ replace soak |

**Exit:** `brain-feed-check.sh` GREEN and soak still honest.

### Ring 2 — Intelligence suggestions (overnight)

**Owner:** Arena / calibration jobs **outside** `gzmo-daemon` job set by default.

| Action | Artifact | Promote rule |
|--------|----------|--------------|
| Arena night | `data-next/arena/latest.json` + champion suggestion | Human pin only (`brain-intel-promote.sh`) |
| Calibration fuse | sibling fused toml | `gzmo config promote-fused --diff` then `--apply` |
| IpW / Forge | recommend under `data-next/arena/forge/` | `blocks_distill=false`; human only |

**Exit:** suggestion present + documented pin decision (accept / defer / reject). Never silent toml overwrite of living overnight.

### Ring 3 — Lab → production ladder (weekly)

**Owner:** Little Tools Lab + GZMO beat-gate kit.

| Stage | Meaning | Gate |
|-------|---------|------|
| S0 Extract | Tool exists with fixture CLI | `cargo test` / fixture exit 0 |
| S1 Mature | Real algorithm + golden fixtures | LTL maturity = mature |
| S2 Beat | Lab recipe ≥ legacy baseline | `beat-gate.sh --loop … --fixture` |
| S2b Live smoke | Optional live vault | `--live` + `VAULT_PATH` to data-next / living probe |
| S3 Handoff | Config/recipe into GZMO path | `gzmo-handoff.sh` / promote-fused — **human** |
| S4 Cutover | Only with `CUTOVER_APPROVED=1` | Explicit; rare |

Loops: `config | ops | cognition | knowledge | discovery` (+ kit extras: pedagogy, ingest, kg).

```bash
# Weekly kit (writes data-next/beat-gate/)
bash scripts/beat-gate-kit.sh --loops config,cognition,knowledge,discovery
# Or full:
bash scripts/beat-gate-kit.sh --all
```

**Exit:** all target loops PASS fixture; any FAIL opens a fix PR before promote.

### Ring 4 — Portfolio craft (monthly)

**Owner:** uniqueness tier list + organ-audit.

1. Re-scan S/A kernels (rarity × craft) — update canvas.
2. Pick **1–2** S/A upgrades (not a portfolio tour).
3. Prefer upgrades that increase Brain Feed nutrient or beat-gate honesty.
4. Run `organ-audit` if autonomic / inbox ownership changes.
5. Re-tier; demote theater that snuck into S.

**Priority queue (seed from 2026-07-21 scan):**

| Priority | Kernel | Upgrade thrust |
|----------|--------|----------------|
| P0 | `spark_field` refractory soft-pick | Harden metrics + beat cognition loop |
| P0 | honeypot `classify_truth_pair` | Golden contradiction fixtures → beat knowledge |
| P0 | herdr OSC arbiter | Living enqueue proof; takeaway side-effect only |
| P1 | HSP dialect MIDI | Keep demable theater; optional motif schema only |
| P1 | Obolus `beats_with_mad` | Feed Arena suggestion quality — still human pin |
| P1 | Thought Cabinet crystallize | Lab chaos only unless beat-gate cognition demands |

---

## Cadence calendar

| When | Ritual | Fail action |
|------|--------|-------------|
| **Daily** | Real work → takeaway; `brain-feed-check.sh` | Fix P0 satellite; no new theater |
| **Nightly** | Living soak + Arena suggestion (if scheduled) | HOLD unpark; do not auto-pin |
| **Weekly** | `beat-gate-kit.sh` + review FAIL loops | Fix PR before any handoff |
| **Weekly** | Promote-fused / serendipity review hour | Accept ≤3; defer rest |
| **Monthly** | Uniqueness re-tier + 1–2 craft upgrades | Ship PR; update canvas |
| **Per merge** | `production-readiness-gate.sh` | Block merge if lite/living boundaries break |

---

## Decision tree — “should this upgrade land?”

```text
Is it Brain Feed P0/P1 or beat-gate honesty?
  NO → demable theater or lab only; do not claim strengthen
  YES ↓
Does it require a second overnight writer?
  YES → redesign as side-effect / suggestion; stop
  NO ↓
Does it auto-swap model/toml/daemon jobs?
  YES → convert to suggestion + human pin; stop
  NO ↓
Fixture beat-gate green for affected loop?
  NO → fix lab first
  YES ↓
Human reviewed artifact + dual-writer check?
  NO → wait
  YES → promote (narrow diff; never full-clobber living)
```

---

## Workstreams (90-day shape)

### W1 — Stabilize the bar (weeks 1–2)

- Keep soak GREEN; Brain Feed P0 all honest.
- Wire weekly `beat-gate-kit.sh` into operator habit (artifact under `data-next/beat-gate/`).
- Freeze uniqueness S-tier list as craft backlog (no new S without demoting one).

### W2 — Nutrient density (weeks 3–6)

- Raise Felt Use depth floor; serendipity promote-back applied weekly (capped).
- herdr close-ritual → living enqueue without memory gym.
- tinyFolder → living path verified; organ-audit clean on Autonomic.

### W3 — Lab honesty (weeks 5–8)

- Cognition + knowledge beat-gates fixture GREEN with versioned baselines.
- Port or deepen honeypot lifecycle / spark refractory tests into LTL goldens.
- Discovery loop smoke on findings → honeypot-gate (CT101 untouched).

### W4 — Suggestion quality (weeks 7–12)

- Arena night → champion suggestion boringly reliable.
- Calibration promote-fused used for real pins (≥1 accepted, ≥1 rejected with reason).
- IpW/Forge only after Arena pin is routine.

---

## Anti-goals

- Mass-publicize private repos as a substitute for local craft.
- Auto-promote Arena champions into living overnight.
- Expanding Unpark theater waves while Brain Feed is red.
- “Upgrade everything” sprints that skip beat-gate and soak.
- Dual-writer “just this once” daemon starts on workstation.

---

## Operator cheat sheet

```bash
# Ring 1
bash scripts/brain-feed-check.sh
bash scripts/keep-quality-soak.sh --summary

# Ring 2
bash scripts/arena-night.sh                    # if scheduled
bash scripts/brain-intel-promote.sh            # suggestion review
gzmo config promote-fused --diff               # then --apply if accepted

# Ring 3
bash scripts/beat-gate-kit.sh --loops config,cognition,knowledge,discovery
# META under data-next/beat-gate/

# Ring 4
# Re-open uniqueness canvas; pick 1 S/A kernel; PR; re-score

# Always after merge
bash scripts/production-readiness-gate.sh
bash scripts/ct101-living-probe.sh             # soft; dual-writer check
```

## Related

- [STACK_OPPORTUNITY_MAP.md](./STACK_OPPORTUNITY_MAP.md)
- [OBOLUS_ARENA_BOUNDARY.md](./OBOLUS_ARENA_BOUNDARY.md)
- [CT101_BOUNDARY.md](./CT101_BOUNDARY.md)
- [SPINE_FOCUS.md](./SPINE_FOCUS.md)
