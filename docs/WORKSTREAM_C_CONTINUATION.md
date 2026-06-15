# Workstream C — Low-Tension Dialogue (Continuation Handoff)

**For:** next worker session (compact — avoids quoting out)  
**Repo:** `survey_GZMO`  
**Parent guide:** [`INFRASTRUCTURE_REMEDIATION_IMPLEMENTATION_GUIDE.md`](./INFRASTRUCTURE_REMEDIATION_IMPLEMENTATION_GUIDE.md) §6

---

## Status snapshot

| Step | What | Status |
|------|------|--------|
| **C1** | Config: `threshold=18`, `idle_ticks_threshold=120` | **Done** (`gzmo.toml`) |
| **C2** | Idle-plateau secondary trigger in watcher | **Done** (`low_tension_dialogue.rs`) |
| **C3** | KG-aware `build_opening` | **Done** (`low_tension_opening.rs` + wired in watcher) |
| **C4** | Persist dialogues to Neo4j | **Not started** (optional) |
| **C5** | Discovery script reads `GZMO_LOW_TENSION_OPENING` | **Done** (`auto-socratic-discovery-cycle.sh` + `pi-mentor-discovery-cycle.sh`) |

---

## What was broken before this handoff

1. `idle_ticks_threshold` existed in config but watcher only fired on **downward crossing** — missed τ plateaus at 17–21%.
2. `build_opening()` was implemented but **never called** — all fires used static template.
3. Only **1** line in `data/pedagogy/low_tension_dialogue.jsonl` (tick 582).

---

## How it works now

**Watcher** (`gzmo-cli/src/low_tension_dialogue.rs`, 5s poll):

- **Primary trigger:** τ crosses from `>= threshold` to `< threshold`
- **Secondary trigger:** `Phase::Idle` AND τ `< threshold` for `idle_ticks_threshold` consecutive polls (120 ≈ 10 min)
- **Guards:** cooldown 300s, `auto_triggers_enabled`, not `ops_mode`
- **Opening:** `gzmo_core::pedagogy::build_opening()` using prerequisite graph + vault recent facts + learner profile; falls back to template
- **Action:** spawns `gzmo_skills/scripts/auto-socratic-discovery-cycle.sh` with env `GZMO_LOW_TENSION_OPENING`

**Config** (`gzmo.toml`):

```toml
[pedagogy.low_tension_dialogue]
enabled = true
threshold = 18.0
cooldown_secs = 300
discovery_cycle = true
idle_ticks_threshold = 120
```

---

## Verify (5 min)

```bash
cd /home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO

# 1. Unit tests
cargo test -p gzmo-cli low_tension

# 2. Rebuild + restart daemon (human operator)
cargo build --release -p gzmo-cli
# kill existing daemon, then: ./target/release/gzmo daemon &

# 3. Confirm watcher logged new fields
rg 'idle_ticks_threshold|Low-tension Socratic watcher' logs/  # or journal

# 4. Wait ~10 min with τ < 18 and phase Idle, OR temporarily lower threshold:
#    threshold = 21.0  idle_ticks_threshold = 12   # ~1 min test window

# 5. Check spawn log
tail -5 /home/maximilian-wruhs/gzmo_skills/data/pi-mentor-discovery/logs/auto-socratic-spawn.log

# 6. Eventually new JSONL lines (bare teach path only; discovery_cycle=true skips JSONL)
wc -l data/pedagogy/low_tension_dialogue.jsonl
```

**Pass criteria:**

- [ ] Daemon log shows `trigger=calm_plateau` or `trigger=crossed_below`
- [ ] `auto-socratic-spawn.log` gains new PID entries
- [ ] Opening text references graph concepts when graphs + unmastered nodes exist

---

## Remaining: C4 (Neo4j persistence) — optional, ~4h

Only do this if JSONL + discovery reports are not enough for recall variety.

1. After successful `teach_autonomous` in `low_tension_dialogue.rs`, call MCP or internal helper to create:
   - Entity: `SOCRATIC_DIALOGUE`
   - Relations: `DIALOGUE_ABOUT` → CONCEPT, `DIALOGUE_WITH` → learner id

2. On `build_opening`, query Neo4j for prior `SOCRATIC_DIALOGUE` about same concept; vary question stem.

3. **Files:** `low_tension_dialogue.rs`, new `gzmo-cli/src/low_tension_persist.rs` (thin wrapper around memory MCP tools).

**Skip C4** if C2+C3+C5 verification passes and discovery cycles produce varied reports.

---

## Tuning cheatsheet

| Symptom | Knob |
|---------|------|
| Never fires (τ stays 18–21%) | `threshold = 21` or lower `idle_ticks_threshold` to 24 (~2 min) |
| Fires too often | Raise `cooldown_secs` or `idle_ticks_threshold` |
| Same generic question | Ensure `data/pedagogy/graphs/*.yaml` loaded; check learner `mastery_vectors` |
| Discovery but no mentor turn | Expected with `discovery_cycle=true` — check `gzmo_skills/data/pi-mentor-discovery/reports/` |

---

## Files touched in C1–C3

| File | Role |
|------|------|
| `gzmo.toml` | threshold, idle_ticks_threshold |
| `gzmo-core/src/config.rs` | `LowTensionDialogueConfig` |
| `gzmo-core/src/pedagogy/low_tension_opening.rs` | KG-aware prompt builder |
| `gzmo-cli/src/low_tension_dialogue.rs` | watcher triggers + wiring |

---

*Stop here unless C4/C5 explicitly requested — Workstreams D–F are separate.*
