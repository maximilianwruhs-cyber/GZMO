# Chaos Engine ρ Homeostasis — Final Revision Report

**Date:** 2026-06-08  
**Scope:** `gzmo-chaos` ρ drift, `LIMIT_CYCLE_BLUEPRINT.md` audit, `chaos-breathing-lab` simulation, production port  
**Repository:** `_foundation-audit/survey_GZMO` (remote: `maximilianwruhs-cyber/GZMO`)  
**Status:** Implementation in working tree — **not committed**; **validated** (unit tests, discrete sim, live CLI)

---

## 1. Executive summary

An agent-produced design doc ([`LIMIT_CYCLE_BLUEPRINT.md`](../gzmo-chaos/LIMIT_CYCLE_BLUEPRINT.md)) correctly identified **one-way `lorenz_rho_mod` accumulation** as a long-horizon problem, but conflated it with **Lorenz ODE collapse** (false) and proposed an unbuilt **limit-cycle ODE** (over-engineered).

Work proceeded in three phases:

1. **Audit** — separated fact from hallucination in the blueprint.
2. **Isolation** — built [`chaos-breathing-lab`](../../../chaos-breathing-lab/) (discrete simulator, zero changes to running GZMO).
3. **Port** — applied the winning policy to `survey_GZMO` and mirrored TypeScript in `survey_edge-node`.

**Winner policy (simulation):** per-tick multiplicative decay `k = 0.001` on `lorenz_rho_mod`, plus joke crystallization cooling `ρ − 0.2`.

**What was NOT implemented:** Inhale/Exhale cycle phase, limit-cycle ODE, Synapse chaos heartbeat (still deferred — daemon does not run `PulseLoop`).

**Architecture (Path A chosen):** In-loop chaos — `PulseLoop` drives LLM params. `rho_effective`, `rho_mod_delta`, `rho_forcing_sign` in `ChaosSnapshot`. See [`CHAOS_RHO_CONTROL_MODEL.md`](CHAOS_RHO_CONTROL_MODEL.md) §8.

**Validation:** Policy passed discrete simulation under `active_story_30s` (max ρ ≈ 5.95, no clamp). **Live CLI execution** confirmed: ported binary compiles, runs, and produces expected ρ regulation under active use (operator-verified).

---

## 2. Problem statement (verified)

### 2.1 Original bug (pre-port)

| Mechanism | Behavior |
|-----------|----------|
| Crystallization | Quote/poem/story/persona added **positive-only** ρ deltas |
| Application | `rho = 28.0 + lorenz_rho_mod` every 174 BPM tick |
| Safeguard | Hard clamp `lorenz_rho_mod ∈ [-10, 10]` → effective ρ ∈ [18, 38] |
| Rebirth | Energy resets; **mutations persist** |
| Symptom | Under active `/story` use, ρ saturates at +10 in ~10 min; normalized Lorenz outputs clip → LLM temperature/tokens lose dynamic range |

### 2.2 What was NOT the bug (blueprint hallucination)

| Claim | Reality |
|-------|---------|
| Lorenz ODE “collapses to fixed point” | Integration stays bounded (`lorenz_stays_bounded` test); attractor remains chaotic in ρ band |
| “Suicide machine” via attractor death | **Death** is `energy <= 0` in [`engine.rs`](../gzmo-chaos/src/engine.rs); orthogonal to ρ |
| Dice “Rho decays by 0.3” | Flavor text only — no mechanical hook |
| Limit cycle in (x,y,z) phase space at ρ ≈ 28±10 | Lorenz native limit cycles occur at very different ρ; wrong regime |

---

## 3. Blueprint audit verdict

Full audit: conversation plan + annotated [`LIMIT_CYCLE_BLUEPRINT.md`](../gzmo-chaos/LIMIT_CYCLE_BLUEPRINT.md).

| Blueprint claim | Verdict |
|-----------------|---------|
| Stories increase ρ via crystallization | **TRUE** (pre-port; still true for story/persona) |
| No ρ decay existed | **TRUE** (pre-port) |
| `thoughts.rs` is accumulation source | **TRUE** |
| Lorenz collapse / suicide machine | **FALSE** — conflated clamp + energy death |
| Inhale/Exhale / Synapse heartbeat | **FICTIONAL** — not built |
| Limit-cycle ODE as required fix | **UNVERIFIED** — simpler decay sufficient in sim |

**Disposition:** Treat blueprint as **partially correct bug report**, not implementation spec.  
**Engineering canon:** [`CHAOS_RHO_CONTROL_MODEL.md`](CHAOS_RHO_CONTROL_MODEL.md) — mathematical terminology replaces mythological framing.

---

## 4. Lab methodology (`chaos-breathing-lab`)

**Location:** `/home/maximilian-wruhs/Projects/chaos-breathing-lab`  
**Purpose:** Simulate Thought Cabinet + ρ policies without modifying `survey_GZMO`.

### Scenarios

| ID | Description |
|----|-------------|
| `idle_auto_lore` | Lore every 30 ticks, 18% absorb, 40% quotes |
| `active_story_30s` | Idle + `/story` every 87 ticks (~30 s) |
| `saturation_recovery` | Burst 5 stories, then idle |

### Policies tested

| Policy | k | Notes |
|--------|---|-------|
| `baseline` | 0 | Legacy one-way accumulation |
| `linear_decay` | 0.0002 | ~20 min half-life |
| `linear_decay_fast` | **0.001** | **~4 min half-life — winner** |
| `nonlinear_decay` | 0.0002 + α=0.15 | Still saturated in active scenario |
| `linear_decay_rebirth` | 0.0002 + 50% halving | Insufficient alone for active chat |

### Matrix results (10,000 ticks)

| Scenario | Policy | max ρ | final ρ | clamp? | pass max&lt;7 |
|----------|--------|-------|---------|--------|-------------|
| `active_story_30s` | baseline | 10.00 | 10.00 | **Y** | n |
| `active_story_30s` | linear_decay | 10.00 | 9.91 | n | n |
| `active_story_30s` | **linear_decay_fast** | **5.99** | **5.74** | n | **Y** |
| `active_story_30s` | nonlinear_decay | 10.00 | 9.78 | n | n |
| `saturation_recovery` | linear_decay_fast | 2.27 | 0.00 | n | Y |

### Single-run confirmation (user hardware)

```
active_story_30s + baseline      → clamp tick 1779 (~10.2 min), max ρ = 10
active_story_30s + linear_decay_fast + joke-cooling → max ρ = 5.95, no clamp
```

Raw artifacts: `chaos-breathing-lab/output/matrix/matrix_summary.tsv`

---

## 5. Implementation ported to GZMO

**Git status at report time:** Changes in `gzmo-chaos/` + `gzmo.toml` are **uncommitted** on branch `main` (HEAD `4f2320e`).

### 5.1 Rust (`gzmo-chaos`)

| File | Change |
|------|--------|
| [`pulse.rs`](../gzmo-chaos/src/pulse.rs) | `ChaosConfig.rho_decay_k` (default `0.001`); `cabinet.apply_rho_decay()` after each cabinet tick |
| [`thoughts.rs`](../gzmo-chaos/src/thoughts.rs) | `apply_rho_decay(k)`; joke crystallization: gravity −0.1, **ρ −0.2**; unit tests |
| [`gzmo.toml`](../gzmo.toml) | `rho_decay_k = 0.001` under `[chaos]` |
| [`gzmo.toml.example`](../gzmo.toml.example) | Documented field; `0.0` disables legacy behavior |

### 5.2 TypeScript mirror (`survey_edge-node`)

| Path | Change |
|------|--------|
| `extensions/chaos-engine/src/{types,thoughts,pulse,index}.ts` | `rhoDecayK`, `applyRhoDecay()`, `joke_cooling` |
| `gzmo-daemon/src/{types,thoughts,pulse}.ts` | Same |

### 5.3 Tick order (important for reviewers)

Per `PulseLoop` iteration:

1. Lorenz step + `apply_rho_mutation(current lorenz_rho_mod)` — uses ρ from **previous** tick
2. Thought Cabinet `tick()` → crystallizations may add/subtract ρ
3. `apply_rho_decay(rho_decay_k)` — multiplicative decay
4. Next tick sees updated ρ

One-tick lag between decay and Lorenz application is intentional and matches the lab simulator.

### 5.4 Disable / rollback

Set in `gzmo.toml`:

```toml
[chaos]
rho_decay_k = 0.0   # disables per-tick decay; joke −0.2 ρ still applies
```

Full legacy crystallization (jokes do not touch ρ): revert `thoughts.rs` joke branch to gravity-only.

---

## 6. What “homeostasis” means now (precise)

**Implemented:**

- Passive per-tick decay: `lorenz_rho_mod *= (1 - k)`
- One bidirectional semantic path: jokes cool ρ
- Existing hard clamp as safety rail (unchanged)

**Not implemented:**

- Nonlinear limit-cycle ODE
- Inhale/Exhale observability (`rho_trend` not in `ChaosSnapshot`)
- Negative ρ paths for quote/story/persona
- Rebirth mutation reset (simulated in lab only; not ported)
- Synapse chaos events (daemon still sets `chaos_feedback_tx: None`)
- Dice flavor “ρ decays by 0.3” wired to mechanics

**Still one-way for dominant chat paths:**

| Category | ρ delta |
|----------|---------|
| quote | +0.3 |
| poem | +0.1 |
| story | +0.5 |
| persona | +0.8 |
| joke | **−0.2** (new) |

Homeostasis is **leaky-integrator + partial semantic cooling**, not full bidirectional crystallization.

---

## 7. Validation status (revision-critical)

| Layer | Status | Evidence |
|-------|--------|----------|
| Unit tests | **PASS** | `cargo test -p gzmo-chaos` — 14 tests incl. `joke_cools_rho`, `rho_decay_halves_over_half_life` |
| Discrete simulation | **PASS** | `chaos-breathing-lab` matrix + hardware single runs (max ρ ≈ 5.95, no clamp under `linear_decay_fast`) |
| Live CLI execution | **PASS** | Ported `gzmo-cli` binary compiles and runs; ρ homeostasis (decay + joke cooling) produces expected regulation metrics under live execution (operator-verified) |
| Daemon integration | **N/A** | `PulseLoop` not in daemon |
| TypeScript runtime | **NOT TESTED** | Mirror port only; edge-node deploy unverified |

**Summary:** Multiplicative decay (`rho_decay_k = 0.001`) plus joke cooling is **verified and functional** through unit tests, discrete simulators, and live CLI. Daemon and TypeScript paths remain out of scope for this validation.

---

## 8. Open items for next revision

| Priority | Item |
|----------|------|
| P0 | **Commit** port with message referencing this report |
| ~~P0~~ | ~~Live CLI verification~~ — **done** (operator-confirmed) |
| P1 | Sweep `k ∈ {0.0004, 0.0005, 0.0007}` if 0.001 feels too fast (~4 min half-life) |
| P1 | Update [`LIMIT_CYCLE_BLUEPRINT.md`](../gzmo-chaos/LIMIT_CYCLE_BLUEPRINT.md) audit header (still says “no ρ decay”) |
| P2 | Add `rho_trend` to `ChaosSnapshot` for observability |
| P2 | Rebirth 50% ρ decay (lab candidate, not ported) |
| P3 | Synapse heartbeat — blocked on daemon `PulseLoop` |
| Arch | Reconcile with [`gzmo-rebuild`](../../../gzmo-rebuild/README.md) lesson: chaos inside inference loop vs scheduler-only spontaneity |

---

## 9. Revision checklist (for reviewers)

- [ ] Read §2 — agree problem statement matches observed saturation
- [ ] Read §3 — blueprint hallucinations documented; limit-cycle scope rejected
- [ ] Read §4 — accept sim evidence or re-run `cargo run -- --matrix -n 10000` in lab
- [ ] Inspect [`thoughts.rs` L145–153, L271–277](../gzmo-chaos/src/thoughts.rs) — joke cooling + decay exist
- [ ] Inspect [`pulse.rs` L106–108, L386](../gzmo-chaos/src/pulse.rs) — config + per-tick call
- [ ] Confirm `gzmo.toml` `[chaos].rho_decay_k` matches intended deployment
- [ ] Run `cargo test -p gzmo-chaos` and `cargo build -p gzmo-cli`
- [x] Live CLI verification — ρ regulation confirmed under ported binary
- [ ] Decide: commit as-is, tune k, or set `rho_decay_k = 0.0` for legacy behavior
- [ ] TS mirror reviewed if edge-node deploys chaos-engine

---

## 10. File manifest

| Path | Role |
|------|------|
| [`docs/CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md`](CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md) | **This document** |
| [`gzmo-chaos/LIMIT_CYCLE_BLUEPRINT.md`](../gzmo-chaos/LIMIT_CYCLE_BLUEPRINT.md) | Original design draft + audit notes (partially stale post-port) |
| [`gzmo-chaos/src/thoughts.rs`](../gzmo-chaos/src/thoughts.rs) | Crystallization + `apply_rho_decay` |
| [`gzmo-chaos/src/pulse.rs`](../gzmo-chaos/src/pulse.rs) | `rho_decay_k` config + loop integration |
| [`gzmo-chaos/src/engine.rs`](../gzmo-chaos/src/engine.rs) | Real death/rebirth (unchanged) |
| [`gzmo.toml`](../gzmo.toml) | `rho_decay_k = 0.001` |
| [`chaos-breathing-lab/`](../../../chaos-breathing-lab/) | Isolated simulator + `RESULTS.md` |
| [`chaos-breathing-lab/output/matrix/`](../../../chaos-breathing-lab/output/matrix/) | CSV + TSV simulation artifacts |

---

## 11. Recommended commit message (when ready)

```
feat(chaos): ρ homeostasis via tick decay and joke cooling

Port chaos-breathing-lab winner: rho_decay_k=0.001 (configurable),
joke crystallization −0.2 lorenz_rho_mod. Validated (unit tests, chaos-breathing-lab sim, live CLI); see
docs/CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md.

Does not implement LIMIT_CYCLE_BLUEPRINT limit-cycle ODE or Synapse heartbeat.
```

---

## 12. Architectural note

[`gzmo-rebuild/README.md`](../../../gzmo-rebuild/README.md) argues chaos should not modulate inference state. This port **fixes saturation within the in-loop model** but does not resolve that architectural question. Teams should decide:

- **A)** Keep in-loop chaos with ρ homeostasis (this port), or  
- **B)** Move spontaneity to scheduler-only (`dice-scheduler`) and decouple Lorenz from LLM params.

Both can coexist during transition; this report documents path **A** only.

---

*End of revision report.*
