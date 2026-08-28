# Chaos ρ — Remaining Implementation Handoff (Agent Brief)

> **Status: CLOSED (2026-06-08).** R1–R9 delivered on `main` (9 commits ahead of origin). R2 skipped — production policy is **tanh** (`rho_restore_alpha=0.01`, `rho_restore_beta=0.5`). See `gzmo-chaos/IMPLEMENTATION_PLAN.md` for crate status.

**Audience:** Implementation agent picking up after 2026-06-08 Path A completion.  
**Repos:** `_foundation-audit/survey_GZMO` (Rust), `_foundation-audit/survey_edge-node` (TS), `chaos-breathing-lab` (sim only)  
**Canonical shipped law:** [`CHAOS_RHO_CONTROL_MODEL.md`](../CHAOS_RHO_CONTROL_MODEL.md)  
**Completed work inventory:** [`CHAOS_RHO_IMPLEMENTATION_HANDOFF.md`](CHAOS_RHO_IMPLEMENTATION_HANDOFF.md) §2 + §17 checklist (all checked)

---

## Session opener (paste to agent)

```
You are implementing REMAINING chaos ρ homeostasis tasks only.

Read first:
  docs/CHAOS_RHO_REMAINING_IMPLEMENTATION_HANDOFF.md  (this file)
  docs/CHAOS_RHO_CONTROL_MODEL.md                     (shipped law — do not contradict)

Rules:
- Do NOT re-implement Workstreams A–G core (decay, daemon, EMA, tanh code, /stabilize) — already shipped.
- Do NOT implement V2 power-law restore or strict limit-cycle ODE (lab-negative / wrong target).
- Do NOT integrate Toto-2.0-4m (IMPACT NO — see TOTO_GZMO_IMPACT_RESEARCH_REPORT.md).
- Lab-gate any change to default ρ policy (k, tanh defaults) via chaos-breathing-lab matrix.
- Minimize scope; match existing Rust/TS style.
- Build: unset CARGO_TARGET_DIR && cargo build --release -p gzmo-cli (see §0.3).

Deliver: PR-sized commits per workstream; update gzmo-chaos/IMPLEMENTATION_PLAN.md status when done.
```

---

## 0. Environment & repo state

### 0.1 Paths

| What | Path |
|------|------|
| Rust chaos crate | `gzmo-chaos/src/{pulse,thoughts,feedback,triggers}.rs` |
| CLI bootstrap | `gzmo-cli/src/chaos_bootstrap.rs` |
| Daemon | `gzmo-cli/src/daemon_cmd.rs` |
| Chat slash | `gzmo-cli/src/chat.rs` |
| TUI | `gzmo-cli/src/tui/runner.rs` |
| Operator config | `gzmo.toml` (gitignored), template `gzmo.toml.example` |
| Live artifacts | `data/CHAOS_STATE.json`, `data/HEARTBEAT.md`, `data/Synapse/events.jsonl` |
| Lab | `~/Projects/chaos-breathing-lab/` |

### 0.2 Build trap (mandatory)

Cursor may set `CARGO_TARGET_DIR` to a sandbox cache. The **daemon runs** `./target/release/gzmo` in the project tree.

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
unset CARGO_TARGET_DIR
cargo build --release -p gzmo-cli
cargo test -p gzmo-chaos
```

### 0.3 Daemon restart (after ρ config changes)

```bash
pkill -f './target/release/gzmo daemon'
nohup ./target/release/gzmo daemon >> logs/daemon-restart.log 2>&1 &
# Verify:
grep 'PulseLoop started' logs/daemon-restart.log | tail -1
jq '{tick,rho_breath_phase,mutations}' data/CHAOS_STATE.json
grep 'chaos.rho_telemetry' data/Synapse/events.jsonl | tail -1
```

`daemon_cmd.rs` pins `PulseHandle` via `_chaos_pulse_keepalive` — do not remove.

### 0.4 Uncommitted local changes (housekeeping — R0)

Working tree may contain doc cleanup not yet on `main`:

- Deleted: `gzmo-chaos/LIMIT_CYCLE_*.md`, `docs/CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md`
- Added: `gzmo-chaos/README.md`, `gzmo-chaos/IMPLEMENTATION_PLAN.md`, Toto research docs
- Modified: `docs/LIMIT_CYCLE_SPECS_MATH_MAP.md`, `docs/CHAOS_RHO_CONTROL_MODEL.md`, `docs/README.md`

**Task R0:** Commit doc cleanup as one PR if not already pushed. Do not restore deleted lore files.

### 0.5 Production config gaps (operator `gzmo.toml`)

Current `[chaos]` has `rho_decay_k = 0.001` only. Missing explicit lines (defaults apply in code):

| Field | Code default | Recommended action |
|-------|--------------|-------------------|
| `rho_ema_gamma` | `0.2` | Add to `gzmo.toml` + example for clarity |
| `rho_restore_alpha` | `0.0` (linear) | **Decision required** — see R1 |
| `rho_restore_beta` | `1.0` | Set if enabling tanh |

---

## 1. What is already done (do not redo)

| Capability | Evidence |
|------------|----------|
| Linear decay `k=0.001` | `thoughts.rs`, `pulse.rs`, tests |
| Tanh restore opt-in | `apply_rho_restoration`, `rho_restore_alpha/beta` |
| Joke −0.2 ρ | `thoughts.rs` + test `joke_cools_rho` |
| Telemetry snapshot | `rho_effective`, `rho_mod_delta`, `rho_forcing_sign`, `rho_breath_phase`, `rho_velocity_ema` |
| EMA breath γ=0.2 | `pulse.rs` |
| `/stabilize` + `skill_stabilize.sh` | `chat.rs`, `feedback.rs`, 18 tests |
| Daemon PulseLoop + Synapse | `chaos_bootstrap.rs`, `chaos.rho_telemetry` every 15 ticks |
| Edge TS parity | `survey_edge-node` commits through `56a8cd6` |
| Lab F tanh winner | `tanh_decay` max ρ 0.93 vs `linear_decay_fast` 5.99 |
| Toto evaluation | **IMPACT NO** — closed |

---

## 2. Remaining workstreams (priority order)

### R1 — Production policy decision: linear vs tanh (P0, operator + code)

**Goal:** Pick and document the live restoration policy under story load.

**Context:** Tanh (α=0.01, β=1.0) dominates linear in lab but is **not enabled** in production (`rho_restore_alpha` defaults to 0).

**Steps:**

1. **A/B on hardware** (30 min each):
   - **A:** `rho_decay_k = 0.001`, `rho_restore_alpha` unset/0
   - **B:** `rho_restore_alpha = 0.01`, `rho_restore_beta = 1.0` (linear k ignored when α>0)
2. Run active `/story` session or wait for daemon story crystallizations; watch `/chaos` and `data/CHAOS_STATE.json`.
3. Record max `lorenz_rho_mod`, subjective “mood” responsiveness.
4. Update **operator** `gzmo.toml` with chosen policy.
5. Update `gzmo.toml.example` comments to state which is **recommended default** (justify with lab + live A/B).
6. Restart daemon; Tier 4 verify (§4).

**Acceptance:**

- `gzmo.toml.example` documents chosen production default
- `docs/CHAOS_RHO_CONTROL_MODEL.md` §4 table reflects decision
- Daemon `CHAOS_STATE.json` tick advances; Synapse events continue

**Estimate:** 2 h (mostly observation).

---

### R2 — Tune `k` sweep (P1, lab-gated, optional if R1 picks tanh)

**Goal:** Slower linear “mood” if staying on linear policy (R1-A).

**Only run if R1 chooses linear** and operator wants gentler decay than `k=0.001` (~4 min half-life).

**Steps:**

1. Extend `chaos-breathing-lab` CLI or `policies.rs` to accept custom `k` (or add `LinearDecayCustom(k)`).
2. Sweep `k ∈ {0.0004, 0.0005, 0.0007, 0.001}` on `active_story_30s` with joke cooling:
   ```bash
   cd ~/Projects/chaos-breathing-lab
   cargo run --release -- --scenario active_story_30s --policy linear_decay_fast -n 10000
   ```
3. Pick **smallest k** with `max_rho < 7` and no clamp.
4. If better than 0.001, update `default_rho_decay_k()` in `pulse.rs`, `gzmo.toml.example`, edge `types.ts`.
5. Append sweep to `chaos-breathing-lab/RESULTS.md`.

**Acceptance:**

- Matrix TSV row for chosen k with `pass_max_lt_7 = Y`
- Tier 1 tests pass

**Estimate:** 2 h.

---

### R3 — TUI parity gaps (P1)

**Goal:** TUI matches chat/daemon ρ surfaces.

**Gaps today:**

| Feature | Chat | TUI |
|---------|------|-----|
| `chaos_bootstrap` | Yes (`chat.rs`) | **No** — inline `PulseLoop::start` in `tui/runner.rs` ~L125 |
| `/stabilize` | Yes | **No** |
| `/chaos` ρ breath row | Yes | Verify canvas/status shows breath |

**Steps:**

#### R3.1 — Refactor TUI to `chaos_bootstrap`

**File:** `gzmo-cli/src/tui/runner.rs`

1. Replace inline `PulseLoop::start` block with:
   ```rust
   let chaos_runtime = crate::chaos_bootstrap::start_chaos_runtime(&config);
   let mut chaos_handle = chaos_runtime.handle; // move into lore task for keepalive
   ```
2. Keep **TUI-specific** snapshot bridge that sends `Action::ChaosSnapshot` (bootstrap bridge does not support this — either:
   - **Option A:** Add optional `on_snapshot: Option<fn(ChaosSnapshot)>` to bootstrap, or
   - **Option B:** Keep a thin local `tokio::spawn` loop cloning `snapshot_rx` like today).
3. Pin `chaos_handle` in a background task (same pattern as current lore receiver).

#### R3.2 — TUI `/stabilize`

Wire palette or slash handler to send `ChaosEvent::Stabilize { delta_rho: -1.0 }` on `chaos_feedback_tx` (grep chat.rs for pattern).

**Acceptance:**

- `cargo build --release -p gzmo-cli` with `unset CARGO_TARGET_DIR`
- TUI runs 2 min; `data/CHAOS_STATE.json` updates
- TUI stabilize decreases ρ within 1 tick (manual check)

**Estimate:** 3–4 h.

---

### R4 — Breath-aware triggers (P2)

**Goal:** `TriggerEngine` reacts to ρ breath phase, not only tension/energy.

**Context:** `TriggerCondition::PhaseEnter` uses `chaos::Phase` (Idle/Build/Drop) — **not** breath. EMA breath is in snapshot but unused by triggers.

**Steps:**

1. **File:** `gzmo-chaos/src/triggers.rs`
   - Add metrics or conditions, e.g.:
     ```rust
     RhoBreathEnter { phase: i8 },  // -1, 0, +1
     // or extend ChaosMetric with RhoMod, RhoVelocityEma
     ```
2. Track `prev_rho_breath_phase` in `TriggerEngine` for edge detection.
3. Add 1–2 default rules in `TriggerEngine::with_defaults()` (whisper on sustained exhale, urgent on ρ_mod > 6 if metric added).
4. Unit test: synthetic snapshot sequence fires expected trigger.

**Acceptance:**

- Test in `triggers.rs` `#[cfg(test)]`
- Daemon log or trigger notify shows fired rule under synthetic load

**Estimate:** 3 h.

---

### R5 — Stabilize configurability (P2)

**Goal:** Tunable stabilize impulse; optional temporary decay boost.

**Partially shipped:** hardcoded `delta_rho: -1.0` in `chat.rs` L902.

**Steps:**

1. Add to `ChaosConfig`:
   ```toml
   [chaos]
   stabilize_delta_rho = -1.0
   ```
2. Wire `chat.rs`, `skill_stabilize.sh` message, edge `feedback.ts`.
3. **Optional extension:** `StabilizeMode::BoostDecay { factor: 5.0, ticks: 174 }` — set `k_eff = 5*k` for M ticks in `PulseLoop` state. Lab not required; document behavior.

**Acceptance:**

- Changing `stabilize_delta_rho = -2.0` in `gzmo.toml` reflects in `/stabilize` effect
- Unit test: stabilize delta from config

**Estimate:** 2 h (+2 h if boost mode).

---

### R6 — Rebirth ρ halving (P3, lab-gated)

**Goal:** On energy rebirth, halve `lorenz_rho_mod` (lab policy `linear_decay_rebirth`).

**Steps:**

1. Confirm lab benefit: `chaos-breathing-lab` `linear_decay_rebirth` row in `matrix_summary.tsv`.
2. **File:** `gzmo-chaos/src/pulse.rs` or `engine.rs` — on rebirth event, `lorenz_rho_mod *= 0.5`.
3. Mirror in edge `pulse.ts`.
4. Test: rebirth reduces ρ.

**Acceptance:**

- Test documents halving
- No regression in Tier 1

**Estimate:** 2 h.

**Skip if:** operator does not use rebirth-heavy sessions.

---

### R7 — Edge-node operator verification (P2, hardware)

**Goal:** Confirm TS chaos-engine matches Rust on a real edge deploy.

**Code parity:** done. **Runtime verify:** not done.

**Steps:**

1. Deploy `survey_edge-node` extension on target Pi/edge stack.
2. Run parallel story load; compare `rhoMod`, `rhoBreathPhase`, decay curves to Rust `CHAOS_STATE.json`.
3. Document pass/fail in `survey_edge-node/README` or one-line in `IMPLEMENTATION_PLAN.md`.

**Acceptance:**

- Written verify note with date + host

**Estimate:** 2–4 h (environment-dependent).

---

### R8 — Observability polish (P3)

**Goal:** Make restore mode visible to operators.

**Steps:**

1. `/chaos` panel: show active restore mode (`linear k=…` vs `tanh α=… β=…`) from `ChaosConfig`.
2. `HEARTBEAT.md` template in `chaos_bootstrap.rs`: add row `Restore policy | linear|tanh`.
3. Optional: log one-line info at PulseLoop start with resolved policy.

**Acceptance:**

- `/chaos` and HEARTBEAT show policy after R1 decision

**Estimate:** 1 h.

---

### R9 — Stale doc repair in parent handoff (P3)

**File:** `docs/CHAOS_RHO_IMPLEMENTATION_HANDOFF.md`

§2.6 still lists Synapse/tanh as NOT shipped — **wrong**. Either:

- Delete §2.6, or
- Replace with short pointer to this file for remaining work.

Also fix §2.1 "apply_rho_decay" → `apply_rho_restoration`; test count 16 → 18.

**Estimate:** 30 min.

---

## 3. Explicitly closed — do not implement

| Item | Reason |
|------|--------|
| V2 power-law \(\mathcal{R}(|\rho|^n)\) | Lab max ρ 9.99 vs linear_fast 5.99 |
| Strict periodic limit cycle | Wrong dynamical target; homeostasis met |
| Schmitt hysteresis Inhale/Exhale | EMA breath shipped; only revisit if EMA fails live |
| Toto-2.0-4m sidecar | `TOTO_GZMO_IMPACT_RESEARCH_REPORT.md` → IMPACT NO |
| Restore `LIMIT_CYCLE_*.md` lore files | Removed; math in `LIMIT_CYCLE_SPECS_MATH_MAP.md` |
| Path B scheduler-only chaos | Architecture decision: Path A |

---

## 4. Verification protocol (run after each workstream)

### Tier 1 — Unit (required)

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
unset CARGO_TARGET_DIR
cargo test -p gzmo-chaos
```

### Tier 3 — Live CLI

```bash
./target/release/gzmo chat   # or existing chat entrypoint
# /chaos  → see ρ_eff, forcing, breath
# /stabilize → ρ_mod drops
```

### Tier 4 — Daemon

```bash
stat data/CHAOS_STATE.json    # mtime advances
jq '.tick' data/CHAOS_STATE.json
grep -c chaos.rho_telemetry data/Synapse/events.jsonl
grep 'ρ breath' data/HEARTBEAT.md
```

**Pass:** tick increments; Synapse events append; HEARTBEAT has ρ rows.

---

## 5. Suggested execution order

```
R0 (docs commit, if needed)
  → R1 (production policy: linear vs tanh)   ← highest leverage
  → R8 (show policy in /chaos + HEARTBEAT)
  → R2 (only if R1 stays linear and k too fast)
  → R3 (TUI parity)
  → R4 (breath triggers)
  → R5 (stabilize config)
  → R7 (edge verify) — parallelizable
  → R6 (rebirth halving) — optional
  → R9 (parent handoff cleanup)
```

---

## 6. Decision tree (remaining only)

```
Start
├─ Need production ρ policy finalized?
│   └─ Yes → R1 (tanh vs linear A/B) → R8
├─ Staying linear but mood too snappy?
│   └─ Yes → R2 (k sweep)
├─ TUI users need stabilize + bootstrap?
│   └─ Yes → R3
├─ Autonomous alerts on breath/exhale?
│   └─ Yes → R4
├─ Edge deploy confidence?
│   └─ Yes → R7
└─ Done → update IMPLEMENTATION_PLAN.md + close agent session
```

---

## 7. Key file touch matrix (remaining only)

| Task | Files |
|------|-------|
| R1 | `gzmo.toml`, `gzmo.toml.example`, `CHAOS_RHO_CONTROL_MODEL.md` |
| R2 | `chaos-breathing-lab/src/policies.rs`, `pulse.rs`, `types.ts`, `RESULTS.md` |
| R3 | `gzmo-cli/src/tui/runner.rs`, possibly `chaos_bootstrap.rs` |
| R4 | `gzmo-chaos/src/triggers.rs`, tests |
| R5 | `pulse.rs`, `chat.rs`, `gzmo.toml.example`, edge `types.ts`, `feedback.ts` |
| R6 | `pulse.rs`, `engine.rs`, edge `pulse.ts` |
| R7 | edge README / `IMPLEMENTATION_PLAN.md` |
| R8 | `chat.rs`, `chaos_bootstrap.rs` |
| R9 | `CHAOS_RHO_IMPLEMENTATION_HANDOFF.md` |

---

## 8. Reference docs (read-only)

| Doc | Use |
|-----|-----|
| [`CHAOS_RHO_CONTROL_MODEL.md`](../CHAOS_RHO_CONTROL_MODEL.md) | Shipped equations |
| [`LIMIT_CYCLE_SPECS_MATH_MAP.md`](../LIMIT_CYCLE_SPECS_MATH_MAP.md) | Lab verdicts, ρ₀=28 correction |
| [`gzmo-chaos/IMPLEMENTATION_PLAN.md`](../../gzmo-chaos/IMPLEMENTATION_PLAN.md) | Crate-local status |
| [`TOTO_GZMO_IMPACT_RESEARCH_REPORT.md`](../reports/TOTO_GZMO_IMPACT_RESEARCH_REPORT.md) | Why no ML forecaster |
| `chaos-breathing-lab/output/matrix/matrix_summary.tsv` | Ground truth numbers |

---

## 9. Sign-off checklist (agent completes when done)

- [x] R1 production policy chosen and documented — **tanh default** (`a79fd6b`)
- [x] R2 skipped or k sweep recorded in `RESULTS.md` — **skipped** (tanh chosen; see `chaos-breathing-lab/RESULTS.md`)
- [x] R3 TUI uses bootstrap + stabilize (`65f6019`)
- [x] R4 breath triggers (`877d066`)
- [x] R5 stabilize config (`d5eda32`)
- [x] Tier 1 + Tier 4 green after changes — `cargo test -p gzmo-chaos` **21 passed**, `cargo build --release -p gzmo-cli` clean
- [x] `IMPLEMENTATION_PLAN.md` updated (`cbc1813`)
- [x] No V2 power-law / Toto / lore restoration introduced

### Delivered commits (`survey_GZMO`)

| Commit | Workstream |
|--------|------------|
| `5f9aae5` | R0 doc cleanup |
| `a79fd6b` | R1 tanh production default |
| `877d066` | R4 breath-aware triggers |
| `65f6019` | R3 TUI parity |
| `d5eda32` | R5 + R6 rebirth halving (Rust) |
| `37201ba` | R8 observability + R9 parent handoff repair |
| `cbc1813` | IMPLEMENTATION_PLAN status |

Edge parity: `survey_edge-node` `e4a4a97` (R6 rebirth halving + R7 verify).

**R2 note:** `k` sweep not run — R1 selected tanh as production policy; linear `rho_decay_k=0.001` remains fallback when `rho_restore_alpha=0`.

---

*Remaining-work handoff — **closed**. Parent inventory: `CHAOS_RHO_IMPLEMENTATION_HANDOFF.md`.*
