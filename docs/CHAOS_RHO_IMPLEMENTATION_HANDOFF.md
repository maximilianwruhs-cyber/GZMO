# Chaos ρ Homeostasis — Implementation Handoff Guide

**Audience:** Engineer continuing Path A (`PulseLoop` in-loop chaos) after the 2026-06-08 survey.  
**Date:** 2026-06-08  
**Repos:** `survey_GZMO` (Rust), `survey_edge-node` (TS mirror), `chaos-breathing-lab` (discrete sim)

---

## 0. How to use this document

| If you need to… | Go to |
|-----------------|-------|
| Understand what shipped | §2 |
| Run verification today | §5 |
| Wire chaos into daemon | §7.1 (Workstream B) |
| Implement MASTER spec phases | §7.2–7.4 |
| Proposal lineage → math | [`LIMIT_CYCLE_SPECS_MATH_MAP.md`](LIMIT_CYCLE_SPECS_MATH_MAP.md) |
| Shipped equations only | [`CHAOS_RHO_CONTROL_MODEL.md`](CHAOS_RHO_CONTROL_MODEL.md) |
| Lab results | `~/Projects/chaos-breathing-lab/RESULTS.md` |

**Rule:** Code and [`CHAOS_RHO_CONTROL_MODEL.md`](CHAOS_RHO_CONTROL_MODEL.md) are **authority**. Mythological drafts were removed from `gzmo-chaos/`.

---

## 1. Executive summary

### Problem (verified)

`lorenz_rho_mod` accumulated positive crystallization deltas with **no per-tick dissipation** → clamp at `+10` → LLM parameter mapping saturates (loss of dynamic range). Not Lorenz ODE blow-up.

### Solution (shipped)

**Discrete leaky integrator** on the accumulator:

\[
\rho_{\mathrm{mod}}[n^+] = \mathrm{clamp}(\rho_{\mathrm{mod}}[n^-] + \sum \Delta\rho_i),\quad
\rho_{\mathrm{mod}}[n+1] = \mathrm{clamp}((1-k)\rho_{\mathrm{mod}}[n^+])
\]

- Default \(k = 0.001\) (`rho_decay_k`)
- Joke crystallization: \(\Delta\rho = -0.2\)
- Telemetry: `rho_effective`, `rho_mod_delta`, `rho_forcing_sign` in `ChaosSnapshot`

### Architecture decision (Path A)

`PulseLoop` runs **inside chat/TUI**, drives Lorenz → gateway LLM overrides. **Not** scheduler-only chaos (`gzmo-rebuild` Path B).

### Git state (as of handoff)

| Repo | Branch | Key commits |
|------|--------|-------------|
| `GZMO` | `main` | `5ff4a94` chaos slice · `0e232a6` platform survey |
| `edge-node` | `master` | `03c28d9` TS ρ mirror |

**Docs cleanup (2026-06-08):** Mythological `LIMIT_CYCLE_*.md` drafts removed from `gzmo-chaos/`. Canonical set: `CHAOS_RHO_CONTROL_MODEL.md`, `LIMIT_CYCLE_SPECS_MATH_MAP.md`, this handoff.

---

## 2. Shipped inventory (do not re-discover)

### 2.1 Rust (`gzmo-chaos`)

| Component | File | What it does |
|-----------|------|--------------|
| Config `rho_decay_k` | `gzmo-chaos/src/pulse.rs` | `ChaosConfig`, default `0.001`, serde from `[chaos]` |
| Tick decay | `gzmo-chaos/src/pulse.rs` | After crystallization: `apply_rho_decay(k)` |
| Decay impl | `gzmo-chaos/src/thoughts.rs` | `(1-k)*ρ_mod`, clamp `[-10,10]` |
| Joke cooling | `gzmo-chaos/src/thoughts.rs` | `crystallize("joke")` → `lorenz_rho_mod -= 0.2` |
| Telemetry | `gzmo-chaos/src/pulse.rs` | `rho_effective`, `rho_mod_delta`, `rho_forcing_sign` |
| Unit tests | `gzmo-chaos/src/thoughts.rs` | `joke_cools_rho`, `rho_decay_halves_over_half_life` |
| Tanh restore (opt-in) | `gzmo-chaos/src/thoughts.rs` | `apply_rho_restoration` when `rho_restore_alpha > 0` |
| Daemon Synapse | `gzmo-cli/src/chaos_bootstrap.rs` | `chaos.rho_telemetry` every 15 ticks |

### 2.2 CLI integration

| Surface | File | Behavior |
|---------|------|----------|
| PulseLoop start | `gzmo-cli/src/chat.rs` ~L112–118 | Loads `[chaos]` TOML → `PulseLoop::start` |
| Gateway coupling | `gzmo-cli/src/chat.rs` ~L127–142 | Snapshot → `set_chaos_overrides(temp, tokens)` |
| `/chaos` panel | `gzmo-cli/src/chat.rs` ~L982–988 | Shows ρ mod, Δ, ρ_eff, forcing sign |
| Feedback ingress | `gzmo-cli/src/chat.rs` | Skills send `ChaosEvent` on `feedback_tx` |
| TUI | `gzmo-cli/src/tui/runner.rs` | Same pattern as chat |

### 2.3 Config

| File | Field |
|------|-------|
| `gzmo.toml.example` | `[chaos].rho_decay_k = 0.001` |
| Local `gzmo.toml` | Gitignored — operator copies from example |

Set `rho_decay_k = 0.0` to restore legacy (no decay).

### 2.4 TypeScript mirror (`survey_edge-node`)

| Path | Parity |
|------|--------|
| `extensions/chaos-engine/src/thoughts.ts` | `applyRhoDecay`, joke −0.2 |
| `extensions/chaos-engine/src/pulse.ts` | decay + telemetry fields |
| `extensions/chaos-engine/src/types.ts` | `rhoDecayK`, snapshot fields |
| `gzmo-daemon/src/*` | Duplicate mirror |

**Gap:** Edge runtime not operator-verified on hardware (code parity only).

### 2.5 Shipped (2026-06-09 execution phase)

| Item | Location |
|------|----------|
| `chaos_bootstrap.rs` | Shared `start_chaos_runtime` + `spawn_snapshot_bridge` |
| Daemon `PulseLoop` | `daemon_cmd.rs` — `chaos_feedback_tx: Some(...)` |
| EMA breath phase | `rho_ema_gamma`, `rho_breath_phase`, `rho_velocity_ema` |
| `/stabilize` | `ChaosEvent::Stabilize`, `skill_stabilize.sh`, `help.rs` |
| HEARTBEAT ρ rows | `chaos_bootstrap.rs` (incl. breath EMA) |
| `LlmGateway::set_chaos_overrides` | `gateway.rs` trait method |

### 2.6 Explicitly NOT shipped

| Item | Blocker / reason |
|------|------------------|
| Synapse `chaos.rho_telemetry` export | Optional Workstream B.5 |
| Tanh / power-law \(\mathcal{R}\) | Lab-negative or unvalidated |
| `engine.rs` rebirth ρ halving | Lab policy `linear_decay_rebirth` — not winner |
| Edge-node EMA + Stabilize | Workstream in progress |

---

## 3. Runtime architecture (one tick)

**Cadence:** 174 BPM → `TICK_INTERVAL` in `pulse.rs` (~345 ms).

```
┌─────────────────────────────────────────────────────────────────┐
│  PulseLoop tick n                                               │
├─────────────────────────────────────────────────────────────────┤
│ 1. Drain ChaosEvent queue (skills, dice, custom) → apply_feedback│
│ 2. Lorenz RK4 step (plant P)                                    │
│ 3. apply_rho_mutation(lorenz_rho_mod)  → ρ = 28 + ρ_mod         │
│ 4. Engine heartbeat (energy, Phase Idle/Build/Drop)  [orthogonal]│
│ 5. ThoughtCabinet.tick() → crystallizations → Δρ impulses       │
│ 6. apply_rho_decay(k)  → (1-k)ρ_mod                             │
│ 7. Compute rho_mod_delta, rho_forcing_sign                        │
│ 8. Map (x,y,z) → llm_temperature, llm_max_tokens, llm_valence   │
│ 9. Broadcast ChaosSnapshot (watch channel)                        │
│10. Every 30 ticks: auto-lore → try_absorb → future crystallize  │
└─────────────────────────────────────────────────────────────────┘
```

**Crystallization Δρ table** (permanent, in `thoughts.rs`):

| Category | Δρ | Other mutations |
|----------|-----|-----------------|
| joke | −0.2 | gravity −0.1 |
| quote | +0.3 | — |
| poem | +0.1 | gravity −0.05 |
| story | +0.5 | — |
| persona | +0.8 | — |
| fact, card, dice_crit, sound | 0 | friction / tension only |

**Name collision:** `chaos::Phase` = `{Idle, Build, Drop}` from **hardware tension**.  
Do **not** reuse for Inhale/Exhale. Use `rho_forcing_sign` or new type `RhoBreathPhase`.

---

## 4. Prerequisites (handoff receiver)

### 4.1 Toolchain

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
rustc --version   # workspace edition 2021
cargo test -p gzmo-chaos
```

### 4.2 Local config

```bash
cp gzmo.toml.example gzmo.toml   # if missing
cp .env.template .env              # if using cloud/neo4j
```

Confirm in `gzmo.toml`:

```toml
[chaos]
rho_decay_k = 0.001
lore_path = "data/lore.toml"
```

### 4.3 Read order (30 minutes)

1. [`CHAOS_RHO_CONTROL_MODEL.md`](CHAOS_RHO_CONTROL_MODEL.md) — 10 min  
2. [`LIMIT_CYCLE_SPECS_MATH_MAP.md`](LIMIT_CYCLE_SPECS_MATH_MAP.md) — 15 min  
3. `chaos-breathing-lab/output/matrix/matrix_summary.tsv` — 5 min  

---

## 5. Verification protocol (step by step)

Run in order. **Do not skip Tier 1** before merging ρ changes.

### Tier 1 — Unit tests (offline, required)

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
cargo test -p gzmo-chaos
```

**Pass criteria:**

- 14+ tests pass including `joke_cools_rho`, `rho_decay_halves_over_half_life`
- `lorenz_stays_bounded` passes

### Tier 2 — Lab matrix (offline, required for policy changes)

```bash
cd ~/Projects/chaos-breathing-lab
cargo test
cargo run -- --matrix -n 10000 --joke-cooling
```

**Pass criteria** (see `output/matrix/matrix_summary.tsv`):

| Scenario | Policy | max ρ | clamp |
|----------|--------|-------|-------|
| `active_story_30s` | `linear_decay_fast` | < 7 | no |
| `saturation_recovery` | any decay + joke | recovers | — |

Record new runs in `chaos-breathing-lab/RESULTS.md`.

### Tier 3 — Live CLI (operator, required for release)

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
cargo build --release
./target/release/gzmo chat   # or your usual chat entrypoint
```

**Steps:**

1. Confirm startup line: `Chaos engine running — 174 BPM`
2. Run `/chaos` — note `Lorenz ρ mod`, `Δ`, `ρ_eff`, `forcing`
3. Run `/story` three times at ~30 s intervals
4. Run `/joke` once
5. Run `/chaos` again

**Pass criteria:**

- `Lorenz ρ mod` stays **below 7** during 5–10 min story session (with `k=0.001`)
- `forcing` shows `+1` after story bursts, `−1` after decay/joke ticks
- `ρ_eff` ≈ `28 + ρ_mod` (e.g. mod 5.7 → eff 33.7)

### Tier 4 — Daemon (verified 2026-06-08)

Requires `PulseHandle` keepalive in `daemon_cmd.rs` (NLL drops `chaos_runtime` early; `Drop` aborts the loop).

- `grep 'PulseLoop started' logs/daemon-restart.log`
- `CHAOS_STATE.json` mtime advances; `tick` increments every ~15 snapshots
- `HEARTBEAT.md` includes `ρ effective`, `ρ mod delta`, `ρ forcing`, `ρ breath (EMA)`
- `data/Synapse/events.jsonl` emits `chaos.rho_telemetry` every 15 ticks (`source: gzmo_daemon`)

### Tier 5 — Edge-node (parity check)

```bash
cd ~/Projects/_foundation-audit/survey_edge-node/extensions/chaos-engine
npm test   # if tests exist
# Run extension in your edge stack; compare snapshot fields to Rust
```

**Pass criteria:** `rhoEffective`, `rhoModDelta`, `rhoForcingSign` track Rust under same story load.

---

## 6. Workstream A — Documentation ✅

**Done:** Math map, control model, handoff on `main`. Lore drafts removed; equations preserved in [`LIMIT_CYCLE_SPECS_MATH_MAP.md`](LIMIT_CYCLE_SPECS_MATH_MAP.md).

---

## 7. Workstream B — Daemon `PulseLoop` integration

**Goal:** Background daemon runs same ρ homeostasis as chat; enable Synapse path.  
**Risk:** Medium (resource lifetime, gateway coupling). **Estimate:** 4–8 h.

### 7.1 Current gap

`gzmo-cli/src/daemon_cmd.rs` builds `OrchestratorContext` with:

```rust
chaos_feedback_tx: None,  // L230-231
```

Chat does (simplified):

```rust
let chaos_config: ChaosConfig = config.chaos.as_ref()...;
let mut chaos_handle = PulseLoop::start(chaos_config);
let chaos_feedback_tx = chaos_handle.feedback_tx.clone();
// spawn: snapshot_rx → gateway + CHAOS_STATE.json + triggers
```

### 7.2 Implementation steps

#### Step B.1 — Extract shared chaos bootstrap

**Why:** Avoid duplicating 80 lines between `chat.rs` and `daemon_cmd.rs`.

**Option A (preferred):** New module `gzmo-core/src/chaos_runtime.rs` or `gzmo-cli/src/chaos_bootstrap.rs`:

```rust
pub struct ChaosRuntime {
    pub handle: PulseLoop::PulseHandle,
    pub feedback_tx: mpsc::Sender<ChaosEvent>,
}

pub fn start_chaos_runtime(config: &GzmoConfig) -> ChaosRuntime { ... }

pub fn spawn_snapshot_bridge(
    snapshot_rx: watch::Receiver<ChaosSnapshot>,
    gateway: Arc<RwLock<Arc<dyn LlmGateway + ...>>>,  // match existing type
    state_dir: PathBuf,
    trigger_notify: Option<mpsc::Sender<String>>,
) -> JoinHandle<()> { ... }
```

**Tasks:**

1. Copy spawn loop from `chat.rs` L127–210 into `spawn_snapshot_bridge`
2. Replace `chat.rs` inline spawn with function call
3. Unit-test: bridge writes `CHAOS_STATE.json` on tick % 15 (temp dir test)

#### Step B.2 — Start PulseLoop in daemon

**File:** `gzmo-cli/src/daemon_cmd.rs`

1. After subsystems init (~L218), before `OrchestratorContext`:
   ```rust
   let chaos_runtime = start_chaos_runtime(&config);
   let chaos_feedback_tx = Some(chaos_runtime.feedback_tx.clone());
   ```

2. Spawn bridge with daemon's gateway (or a dedicated inference gateway if different):
   ```rust
   let chaos_bridge = spawn_snapshot_bridge(
       chaos_runtime.handle.snapshot_rx.clone(),
       Arc::clone(&orch_gateway),  // verify this gateway is used for live inference
       config.memory.vault_db.parent().unwrap_or(Path::new("data")).to_path_buf(),
       None,  // triggers optional in daemon
   );
   ```

3. Pass `chaos_feedback_tx` into `OrchestratorContext` (replace `None`).

4. On daemon shutdown: abort `chaos_runtime.handle.task`, drop bridge handle.

#### Step B.3 — Wire watcher feedback

**File:** `gzmo-core/src/watcher.rs` ~L209

Already sends `ChaosEvent::Custom` **if** `chaos_feedback_tx` is `Some`. Verify after B.2:

- File drop in inbox triggers chaos feedback in **daemon** logs

#### Step B.4 — Orchestrator / skills in daemon mode

Confirm background jobs that invoke shell skills can reach chaos feedback channel. If headless skills run via `skill_dispatch.sh` only (file-based), may need:

- `CHAOS_STATE.json` reader in skills (legacy), or
- IPC channel from skill runner to daemon (future)

**Minimum acceptance:** PulseLoop runs; snapshots written; `chaos_feedback_tx` not None.

#### Step B.5 — Synapse export (optional in same PR)

**Files:** `gzmo-core/src/synapse.rs`, snapshot bridge

On tick % N (e.g. 15), append event:

```json
{
  "kind": "chaos.rho_telemetry",
  "rho_mod": ...,
  "rho_effective": ...,
  "rho_mod_delta": ...,
  "rho_forcing_sign": ...
}
```

**Defer** if Synapse schema unsettled — document in PR.

### 7.3 Verification (Workstream B)

```bash
cargo test
cargo build --release
./scripts/start-production.sh --daemon   # your ops path
# Wait 2 min
cat data/CHAOS_STATE.json | jq '.rho_effective, .rho_mod_delta, .mutations.lorenz_rho_mod'
```

**Pass criteria:**

- `CHAOS_STATE.json` updates every ~15 ticks
- `rho_decay_k` from `gzmo.toml` reflected (set `0.0`, confirm ρ drifts up under forced stories)
- Daemon log: `PulseLoop started — 174 BPM`
- No duplicate PulseLoop (chat + daemon) if both run — **document:** only one should own chaos per process

### 7.4 Rollback

```toml
[chaos]
rho_decay_k = 0.0
```

Or revert daemon commit and keep chat-only Path A.

---

## 8. Workstream C — ρ telemetry in `HEARTBEAT.md`

**Goal:** Human-readable parity with `/chaos` panel.  
**Risk:** Low. **Estimate:** 30 min.

### Steps

1. **File:** `gzmo-cli/src/chat.rs` (and extracted `spawn_snapshot_bridge` if Workstream B done)

2. In heartbeat `format!` block (~L153–184), add rows:
   ```
   | ρ effective | {:.2} |
   | ρ mod delta | {:+.3} |
   | ρ forcing   | {:+} |
   ```
   Use `snap.rho_effective`, `snap.rho_mod_delta`, `snap.rho_forcing_sign`.

3. Rebuild, run chat 2 min, inspect `data/HEARTBEAT.md`.

### Acceptance

- Markdown table shows ρ telemetry
- Tier 3 `/chaos` values match HEARTBEAT within one tick

---

## 9. Workstream D — Tune `k` (optional)

**Goal:** Slower “mood” without clamp under story load.  
**Risk:** Low if lab-gated. **Estimate:** 2 h.

### Steps

1. Lab sweep:
   ```bash
   cd ~/Projects/chaos-breathing-lab
   # Edit src/policies.rs or add CLI flags for k in {0.0004, 0.0005, 0.0007}
   cargo run -- --scenario active_story_30s --policy linear_decay_fast -n 10000 --joke-cooling
   ```

2. Pick smallest \(k\) with `max_rho < 7` and no clamp.

3. Update defaults only if strictly better:
   - `gzmo-chaos/src/pulse.rs` `default_rho_decay_k()`
   - `gzmo.toml.example`
   - `survey_edge-node` `defaultConfig().rhoDecayK`

4. Re-run Tier 1–3 verification.

### Acceptance

- `RESULTS.md` documents sweep
- Matrix TSV attached or cited in PR

---

## 10. Workstream E — EMA breath phase (MASTER Phase II)

**Goal:** Smoothed \(\mathrm{sign}(\dot{\rho})\) for triggers/UI — **not** `chaos::Phase`.  
**Risk:** Low. **Estimate:** 2–3 h.

### Math

\[
v[n] = (1-\gamma)\,v[n-1] + \gamma\,\Delta\rho_{\mathrm{mod}}[n],
\quad \gamma \in [0.1, 0.3]
\]

\[
\text{breath\_phase}[n] = \mathrm{sign}(v[n]) \in \{-1, 0, +1\}
\]

Shipped `rho_forcing_sign` = \(\mathrm{sign}(\Delta\rho)\) with \(\gamma = 1\) (no memory).

### Steps

#### E.1 — Types

**File:** `gzmo-chaos/src/pulse.rs`

```rust
/// Smoothed ρ accumulator tendency. NOT chaos::Phase (Idle/Build/Drop).
pub type RhoBreathPhase = i8;  // -1, 0, +1
```

Add to `ChaosSnapshot`:

```rust
pub rho_breath_phase: RhoBreathPhase,
pub rho_velocity_ema: f64,
```

Add to `ChaosConfig`:

```rust
#[serde(default = "default_rho_ema_gamma")]
pub rho_ema_gamma: f64,  // default 0.2
```

#### E.2 — PulseLoop state

In `PulseLoop` struct:

```rust
rho_velocity_ema: f64,
```

In tick loop after `rho_mod_delta`:

```rust
let gamma = self.config.rho_ema_gamma;
self.rho_velocity_ema = (1.0 - gamma) * self.rho_velocity_ema + gamma * rho_mod_delta;
let rho_breath_phase = if self.rho_velocity_ema > 1e-9 { 1 }
    else if self.rho_velocity_ema < -1e-9 { -1 } else { 0 };
```

#### E.3 — Tests

```rust
#[test]
fn ema_smooths_single_spike() { ... }
```

#### E.4 — CLI `/chaos`

Display `breath_phase (EMA)` alongside `forcing (instant)`.

#### E.5 — Mirror TS

`extensions/chaos-engine/src/types.ts` + `pulse.ts` + `gzmo-daemon/src/*`.

#### E.6 — TriggerEngine (optional)

**File:** `gzmo-chaos/src/triggers.rs`

New condition `RhoBreathPositive` / `RhoBreathNegative` for alerts.

### Acceptance

- Unit test: alternating +/− impulses → EMA lags instant sign
- Tier 3: `/chaos` shows both instant and EMA phase
- No rename of `chaos::Phase`

---

## 11. Workstream F — Tanh governor (MASTER Phase I)

**Goal:** Test bounded nonlinear \(\mathcal{R}\) before any production replace.  
**Risk:** High if merged without lab win. **Estimate:** 4 h lab + 2 h port.

### Correct equation (see math map §2.3)

**Wrong (lore):** \(\tanh(\beta(\rho_{\mathrm{mod}} - 28))\)  
**Right:**

\[
\rho_{\mathrm{mod}}[n+1] = \mathrm{clamp}\!\left(
\rho_{\mathrm{mod}}[n^+] - \alpha\,\tanh(\beta\,\rho_{\mathrm{mod}}[n^+]),
\; [-10,10]\right)
\]

### Steps

#### F.1 — Lab only first

**File:** `chaos-breathing-lab/src/policies.rs`

Add `PolicyKind::TanhDecay { alpha, beta }` and implement `apply_decay`.

```bash
cargo run -- --matrix -n 10000 --joke-cooling
```

**Gate:** `active_story_30s` max ρ < 7 **and** beats or matches `linear_decay_fast`.

If **fail** → stop; document in `RESULTS.md`; do not port.

**Result (2026-06-08):** PASS — `tanh_decay` max ρ=0.93 vs `linear_decay_fast` 5.99. Ported opt-in (`rho_restore_alpha=0` default).

#### F.2 — Rust port (only if F.1 passes)

**File:** `gzmo-chaos/src/thoughts.rs`

```rust
pub fn apply_rho_restoration(&mut self, config: &ChaosConfig) {
    if config.rho_restore_alpha > 0.0 {
        let r = config.rho_restore_alpha * (config.rho_restore_beta * self.mutations.lorenz_rho_mod).tanh();
        self.mutations.lorenz_rho_mod -= r;
    } else {
        self.apply_rho_decay(config.rho_decay_k);
    }
    self.mutations.lorenz_rho_mod = self.mutations.lorenz_rho_mod.clamp(-10.0, 10.0);
}
```

**Config:** `rho_restore_alpha`, `rho_restore_beta`, keep `rho_decay_k` for A/B.

#### F.3 — Verification

Full Tier 1–3 + matrix comparison CSV.

### Acceptance

- Lab matrix documents tanh policy
- PR cites `LIMIT_CYCLE_SPECS_MATH_MAP.md` §2.3
- `linear_decay_fast` remains default if tanh does not win

---

## 12. Workstream G — `skill_stabilize` (MASTER Phase III)

**Goal:** Agent-triggered negative ρ impulse or temporary decay boost.  
**Risk:** Low. **Estimate:** 3 h.

### Math

Manual event: \(\rho_{\mathrm{mod}} \mathrel{+}= \Delta\rho_{\mathrm{stab}}\) with \(\Delta\rho_{\mathrm{stab}} = -1.0\) (tunable), clamped.

Or: set `k_boost = 5k` for \(M\) ticks (e.g. 60 s ≈ 174 ticks).

### Steps

#### G.1 — Feedback event

**File:** `gzmo-chaos/src/feedback.rs`

```rust
Stabilize { delta_rho: f64 },  // default -1.0
```

#### G.2 — Apply in `pulse.rs` `apply_feedback`

```rust
ChaosEvent::Stabilize { delta_rho } => {
    self.cabinet.mutations.lorenz_rho_mod =
        (self.cabinet.mutations.lorenz_rho_mod + delta_rho).clamp(-10.0, 10.0);
}
```

#### G.3 — Skill shell

**File:** `skills/skill_stabilize.sh` (new)

Send event via existing skill → chaos feedback path (match `skill_story.sh` pattern).

#### G.4 — Slash command

**File:** `gzmo-cli/src/chat.rs` `handle_slash_command`

`/stabilize` → `ChaosEvent::Stabilize { delta_rho: -1.0 }`.

#### G.5 — Config (optional)

```toml
[chaos]
stabilize_delta_rho = -1.0
```

### Acceptance

- After `/stabilize`, `/chaos` shows ρ_mod drop ≥ 0.5 within 1 tick
- Unit test: stabilize from ρ=8 → ≤7
- Document in `CHAOS_RHO_CONTROL_MODEL.md` §6

---

## 13. File touch matrix (quick reference)

| Workstream | Primary files |
|------------|---------------|
| A Docs | `docs/LIMIT_CYCLE_SPECS_MATH_MAP.md`, `CHAOS_RHO_CONTROL_MODEL.md`, this file |
| B Daemon | `daemon_cmd.rs`, new `chaos_bootstrap.rs`, `orchestrator.rs`, `watcher.rs` |
| C HEARTBEAT | `chat.rs` or `chaos_bootstrap.rs` |
| D Tune k | `pulse.rs`, `gzmo.toml.example`, `chaos-breathing-lab/`, edge `types.ts` |
| E EMA phase | `pulse.rs`, `chat.rs`, edge `pulse.ts`, `triggers.rs` |
| F Tanh | `chaos-breathing-lab/` first, then `thoughts.rs`, `pulse.rs`, `ChaosConfig` |
| G Stabilize | `feedback.rs`, `pulse.rs`, `skills/skill_stabilize.sh`, `chat.rs` |

---

## 14. Decision tree

```
Need to fix ρ saturation under /story?
├─ Yes → Is rho_decay_k = 0.001 in gzmo.toml?
│        ├─ No → Set it (Tier 3 verify)
│        └─ Yes → Still saturates?
│                 ├─ Yes → Workstream D (tune k) or more negative Δρ categories
│                 └─ No → Done
└─ No → Need daemon background chaos?
         ├─ Yes → Workstream B (mandatory)
         └─ No → Chat-only Path A is complete

Want "breathing" UI phase?
├─ Instant sign → already shipped (rho_forcing_sign)
└─ Smoothed → Workstream E (EMA)

Want nonlinear restore (MASTER / V2)?
└─ Workstream F — lab gate REQUIRED

Want agent override?
└─ Workstream G
```

---

## 15. Rollback & feature flags

| Action | Effect |
|--------|--------|
| `rho_decay_k = 0.0` | Legacy open-loop accumulation |
| Disable joke ρ | Revert `thoughts.rs` joke branch (−0.2 line) |
| Daemon chaos off | `chaos_feedback_tx: None` + do not start PulseLoop |
| Full revert | `git revert 5ff4a94` (chaos slice only) |

---

## 16. Related repositories

| Path | Role |
|------|------|
| `_foundation-audit/survey_GZMO` | Production Rust |
| `_foundation-audit/survey_edge-node` | TS mirror |
| `chaos-breathing-lab` | Policy experiments (no GZMO writes) |
| `gzmo-rebuild` | Path B — **not** primary; do not merge without explicit arch decision |

---

## 17. Handoff checklist (sign-off)

Copy to PR or issue when transferring ownership:

- [x] Read §1–3 of this doc + `CHAOS_RHO_CONTROL_MODEL.md`
- [x] Tier 1 tests pass locally (16 `gzmo-chaos` tests)
- [x] `gzmo.toml` has `rho_decay_k = 0.001`
- [x] Tier 3 live `/chaos` + `/stabilize` verified
- [x] Daemon `PulseLoop` wired (`chaos_feedback_tx: Some`)
- [x] Understand `chaos::Phase` ≠ Inhale/Exhale (`rho_breath_phase` separate)
- [x] Workstream A docs committed (`2d7cdcf`)
- [x] Workstreams B, C, E, G implemented
- [x] Tier 4 daemon verify on production stack (2026-06-08; keepalive fix)
- [x] Edge-node TS parity (EMA + Stabilize)
- [x] Workstream F (tanh) — lab passed; ported opt-in (`rho_restore_alpha`, `rho_restore_beta`)

---

*End of handoff. Update this file when Workstreams B–G land or lab defaults change.*
