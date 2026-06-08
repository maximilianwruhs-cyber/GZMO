# How to Use `gzmo-chaos` — Agent Guide

**Audience:** Any agent starting a fresh session on GZMO chaos / ρ homeostasis.  
**Repo:** `/home/maximilian-wruhs/Projects/_foundation-audit/survey_GZMO`  
**Status (2026-06-08):** Path A + remaining handoff **complete**. Do not re-implement decay, tanh, EMA, `/stabilize`, TUI parity, or breath triggers unless fixing a regression.

---

## Session opener (paste to agent)

```
You are working with gzmo-chaos (Lorenz + Thought Cabinet + ρ homeostasis).

Read first:
  docs/GZMO_CHAOS_AGENT_GUIDE.md          (this file — how to run & extend)
  docs/CHAOS_RHO_CONTROL_MODEL.md       (shipped equations — do not contradict)
  gzmo-chaos/IMPLEMENTATION_PLAN.md     (what is shipped vs deferred)

Rules:
- ONE PulseLoop owner per data/ directory: `gzmo daemon` OR `gzmo chat`/`gzmo` TUI — never both.
- Build: unset CARGO_TARGET_DIR && cargo build --release -p gzmo-cli
- Test:  unset CARGO_TARGET_DIR && cargo test -p gzmo-chaos
- Lab-gate any change to default ρ policy via ~/Projects/chaos-breathing-lab matrix.
- Chaos crystallization ≠ dream consolidation. Chaos does NOT write vault.db.
- Do NOT implement V2 power-law restore, strict limit-cycle ODE, or Toto-2.0-4m.
```

---

## 1. What `gzmo-chaos` is

`gzmo-chaos` is a **deterministic in-process chaos engine** that:

1. Integrates a **Lorenz attractor** at **174 BPM** (one tick ≈ 345 ms).
2. Maps Lorenz coordinates → **LLM temperature / max_tokens / valence**.
3. Runs a **Thought Cabinet**: skill outputs may be absorbed, incubate, then **crystallize** into permanent parameter mutations (`lorenz_rho_mod`, gravity, friction).
4. Applies **ρ homeostasis**: per-tick decay or tanh restoration so `lorenz_rho_mod` stays bounded.

It is **not**:

- DreamEngine / SparkEngine / IngestEngine (those live in `gzmo-core`, run on **daemon cron** or `gzmo dream`).
- A vault writer. Thought Cabinet “crystallization” is **in-memory ρ control only**.
- A second copy of the edge-node TS stack (parity exists in `survey_edge-node` for deploy).

---

## 2. Directory map

| What | Path |
|------|------|
| Chaos crate (ρ law, PulseLoop, cabinet) | `gzmo-chaos/src/{pulse,thoughts,feedback,triggers,chaos,engine}.rs` |
| CLI bootstrap (shared start + bridge) | `gzmo-cli/src/chaos_bootstrap.rs` |
| Daemon owner | `gzmo-cli/src/daemon_cmd.rs` |
| Chat slash commands | `gzmo-cli/src/chat.rs` |
| TUI | `gzmo-cli/src/tui/runner.rs`, `tui/components/agent.rs` |
| Operator config (gitignored) | `gzmo.toml` |
| Config template | `gzmo.toml.example` → `[chaos]` |
| Live snapshots | `data/CHAOS_STATE.json`, `data/HEARTBEAT.md` |
| Daemon telemetry | `data/Synapse/events.jsonl` (`chaos.rho_telemetry`) |
| Lab simulator | `~/Projects/chaos-breathing-lab/` |
| TS parity | `~/Projects/_foundation-audit/survey_edge-node/{gzmo-daemon,extensions/chaos-engine}/` |

---

## 3. How to run (operators)

### Build (mandatory trap)

Cursor may set `CARGO_TARGET_DIR` to a sandbox cache. The running binary is **`./target/release/gzmo`** in the project tree.

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
unset CARGO_TARGET_DIR
cargo build --release -p gzmo-cli
cargo test -p gzmo-chaos    # expect 21 passed
```

### One PulseLoop owner

| Mode | Command | PulseLoop | Synapse ρ telemetry |
|------|---------|-----------|---------------------|
| **Daemon** (background brain) | `./target/release/gzmo daemon` | Yes | Yes (every 15 ticks) |
| **Chat REPL** | `./target/release/gzmo chat` | Yes | No |
| **TUI** | `./target/release/gzmo` (default) | Yes | No |

**Never run daemon + chat/TUI on the same `data/` at once.** They are sibling owners, not nested. Concurrent use causes snapshot fights and SQLite lock waits on shared artifacts — not chaos-specific DB corruption.

### Daemon restart (after `[chaos]` config changes)

```bash
pkill -f './target/release/gzmo daemon'
rm -f /tmp/gzmo_rust.pid   # if stale lockfile blocks start
nohup ./target/release/gzmo daemon >> logs/daemon-restart.log 2>&1 &
sleep 6
grep 'PulseLoop started' logs/daemon-restart.log | tail -1
jq '{tick,rho_breath_phase,mutations}' data/CHAOS_STATE.json
grep 'chaos.rho_telemetry' data/Synapse/events.jsonl | tail -1
```

`daemon_cmd.rs` pins `PulseHandle` via `_chaos_pulse_keepalive` — **do not remove** or the PulseLoop drops on scope exit.

---

## 4. Operator commands

### Slash commands (chat + TUI)

| Command | Effect |
|---------|--------|
| `/chaos` | Print live `ChaosSnapshot`: Lorenz coords, cabinet stats, ρ telemetry, restore policy |
| `/stabilize` | Send `ChaosEvent::Stabilize { delta_rho }` — default **−1.0**, tunable via config |
| `/story`, `/joke`, `/poem`, … | Shell skills → `ChaosEvent` feedback → may seed Thought Cabinet |

`/stabilize` also exists as `skills/skill_stabilize.sh` for shell-skill parity.

### Read live state without chat

```bash
cat data/HEARTBEAT.md          # human table incl. Restore Policy
jq . data/CHAOS_STATE.json     # full snapshot JSON
```

Production default (local `gzmo.toml`): **tanh** restore `α=0.01`, `β=1.0`.

---

## 5. Configuration (`[chaos]`)

```toml
[chaos]
gravity = 9.8
friction = 0.5
seed = 0.506
rho_decay_k = 0.001              # linear restore when tanh disabled
rho_restore_alpha = 0.01         # >0 enables tanh (replaces linear)
rho_restore_beta = 1.0
rho_ema_gamma = 0.2              # EMA for rho_breath_phase
stabilize_delta_rho = -1.0       # /stabilize impulse
```

| Policy | When | Restore law |
|--------|------|-------------|
| **Tanh** (production default) | `rho_restore_alpha > 0` | ρ ← ρ − α·tanh(β·ρ) |
| **Linear** (fallback) | `rho_restore_alpha = 0` | ρ ← (1−k)·ρ per tick |

**Lab-gate** any change to α, β, or k defaults: run `chaos-breathing-lab` matrix before merging.

---

## 6. Architecture for developers

### Data flow

```
PulseLoop (174 BPM)
  ├─ integrate Lorenz ODE (ρ = 28 + lorenz_rho_mod)
  ├─ tick Thought Cabinet → crystallization impulses
  ├─ apply_rho_restoration (linear or tanh)
  ├─ rebirth → lorenz_rho_mod *= 0.5
  └─ broadcast ChaosSnapshot via watch channel

chaos_bootstrap::spawn_snapshot_bridge
  ├─ gateway.set_chaos_overrides(temp, max_tokens)  every tick
  ├─ write CHAOS_STATE.json + HEARTBEAT.md          every 15 ticks
  ├─ Synapse chaos.rho_telemetry                    daemon only
  └─ TriggerEngine::evaluate (breath alerts, ρ critical)
```

### Starting chaos from Rust

```rust
use gzmo_chaos::pulse::{PulseLoop, ChaosConfig};
use gzmo_chaos::feedback::ChaosEvent;

let handle = PulseLoop::start(ChaosConfig::default());
let snap = handle.snapshot_rx.borrow().clone();
handle.feedback_tx.send(ChaosEvent::Stabilize { delta_rho: -1.0 }).await?;
```

Production entry points use `chaos_bootstrap::start_chaos_runtime(&config)` instead of calling `PulseLoop::start` directly (chat, TUI, daemon all use this).

### Key types

| Type | Role |
|------|------|
| `ChaosSnapshot` | Live state: Lorenz, cabinet, `rho_effective`, `rho_mod_delta`, `rho_breath_phase`, LLM params |
| `ChaosEvent` | Skill → engine feedback (`JokeGenerated`, `Stabilize`, `DiceRoll`, …) |
| `CrystallizationEvent` | Cabinet completion → permanent `MutationEffect` on ρ/gravity/friction |
| `TriggerEngine` | Autonomous alerts on breath phase / ρ thresholds |

### Terminology (use engineering names)

| Say | Not |
|-----|-----|
| `lorenz_rho_mod` accumulator | “crystallized dream in vault” |
| `rho_breath_phase` (EMA sign) | `chaos::Phase` (Idle/Active lifecycle) |
| PulseLoop tick | “heartbeat metaphor” in code comments |

Full map: `docs/CHAOS_RHO_CONTROL_MODEL.md` §3.

---

## 7. What touches the database (safety)

| Mechanism | Writes `vault.db`? |
|-----------|-------------------|
| Thought Cabinet crystallization | **No** — in-memory ρ/gravity/friction only |
| Chaos snapshots / HEARTBEAT / Synapse | **No** — files under `data/` |
| DreamEngine consolidation | **Yes** — daemon cron / `gzmo dream` only |
| Chat `memory_record` tool, `/remember`, `/quit` summary | **Yes** — explicit agent/user paths |

Chaos chat **cannot flood the vault with crystallized thoughts**. Vault growth risk is memory tools + dream/spark jobs, not ρ homeostasis.

---

## 8. Verification checklist

After code or config changes:

```bash
unset CARGO_TARGET_DIR
cargo test -p gzmo-chaos                    # 21 tests
cargo build --release -p gzmo-cli
./target/release/gzmo chat                  # /chaos and /stabilize
# or restart daemon and check:
jq '.rho_effective, .rho_breath_phase' data/CHAOS_STATE.json
grep 'Restore Policy' data/HEARTBEAT.md
```

Edge TS parity (if Rust pulse logic changed):

```bash
cd ~/Projects/_foundation-audit/survey_edge-node/gzmo-daemon
npx tsc --noEmit
```

---

## 9. Reading order

| Priority | Document | Why |
|----------|----------|-----|
| 1 | **This file** | Run, extend, safety boundaries |
| 2 | `docs/CHAOS_RHO_CONTROL_MODEL.md` | Shipped equations |
| 3 | `gzmo-chaos/IMPLEMENTATION_PLAN.md` | Shipped vs deferred |
| 4 | `docs/CHAOS_RHO_IMPLEMENTATION_HANDOFF.md` | Full inventory + verify tiers |
| 5 | `docs/LIMIT_CYCLE_SPECS_MATH_MAP.md` | Proposal history + lab verdicts only |
| 6 | `docs/CHAOS_RHO_REMAINING_IMPLEMENTATION_HANDOFF.md` | **Closed** — historical agent brief |

---

## 10. Safe extension targets (if asked)

| Task | Where to change |
|------|-----------------|
| New crystallization impulse | `gzmo-chaos/src/thoughts.rs` + lab matrix |
| New skill feedback event | `feedback.rs` + skill handler in `gzmo-core/src/skills/` |
| New autonomous trigger | `gzmo-chaos/src/triggers.rs` |
| Observability field | `ChaosSnapshot` in `pulse.rs` + `chaos_bootstrap.rs` HEARTBEAT template |
| Production ρ policy | `gzmo.toml` + lab gate + `CHAOS_RHO_CONTROL_MODEL.md` |

## 11. Do not implement (closed / lab-negative)

- V2 power-law \(\mathcal{R}\) restore
- Strict periodic limit-cycle ODE in Lorenz phase space
- Toto-2.0-4m forecaster (`docs/TOTO_GZMO_IMPACT_RESEARCH_REPORT.md` → IMPACT NO)
- Restoring deleted `LIMIT_CYCLE_*.md` lore drafts
- Re-running R1–R9 remaining handoff (see closed brief)

---

## 12. Quick mental model

**Chaos is a mood thermostat on LLM parameters.** Skills and lore create bursts on `lorenz_rho_mod`; decay/tanh pulls it back; `/stabilize` is manual override; rebirth halves accumulated ρ after energy death. The daemon keeps this running 24/7 and feeds Synapse; chat/TUI run the same engine interactively for one session.

---

## 13. Extensive testing — agent prompt

Copy one of the blocks below into a **new agent session**. The agent must **run commands itself** and return a structured report. Do not accept “should pass” without evidence.

### Full regression (no code changes expected)

```
You are a QA agent for gzmo-chaos. Do NOT implement features — test only.

Read:
  docs/GZMO_CHAOS_AGENT_GUIDE.md §8 and §13
  docs/CHAOS_RHO_IMPLEMENTATION_HANDOFF.md §5 (Tiers 1–5)

Repo: ~/Projects/_foundation-audit/survey_GZMO
Always: unset CARGO_TARGET_DIR before cargo commands.

Run ALL tiers in order. Stop and report on first hard failure.

Tier 1 — Unit tests
  cargo test -p gzmo-chaos
  Pass: 21 tests green (incl. joke_cools_rho, rho_decay_halves_over_half_life,
        feedback_stabilize_reduces_rho, test_rebirth_halves_rho, test_rho_breath_trigger)

Tier 2 — Lab matrix (sanity; skip only if no policy change in last 7 days)
  cd ~/Projects/chaos-breathing-lab
  cargo test && cargo run --release -- --matrix -n 10000
  Pass: active_story_30s max ρ < 7 under tanh_decay; compare matrix_summary.tsv

Tier 3 — Live chat stress (ONE owner: no daemon on same data/)
  pkill -f './target/release/gzmo daemon' || true
  # Shell skills need FULL completions URL (gzmo.toml [engine.local].url is base /v1 only)
  export GZMO_LLM_URL=http://localhost:8000/v1/chat/completions
  export GZMO_LLM_MODEL=qwen3.6-35b-mtp   # match [engine.local].model in gzmo.toml
  bash scripts/qa/tier3_stress_chat.sh
  Pass criteria:
    - Startup: "Chaos engine running — 174 BPM"
    - Each /story prints non-empty prose (not LLM/curl errors)
    - Story 1 may show Incubating: 1, ρ mod unchanged (cabinet incubates ~40 ticks)
    - Later stories: crystallization fires; ρ mod moves; forcing/breath may go +1
    - Lorenz ρ mod stays < 7 (with tanh default, expect small |ρ_mod| ≪ 7)
    - /stabilize drops ρ mod by ≈|stabilize_delta_rho| (default 1.0)
    - ρ_eff ≈ 28 + ρ_mod
    - Restore policy line matches gzmo.toml (tanh or linear)

Tier 4 — Daemon + artifacts
  Restart daemon; wait ≥30s
  Verify: PulseLoop started in logs; CHAOS_STATE.json tick advances;
          HEARTBEAT.md has Restore Policy + ρ rows;
          Synapse has chaos.rho_telemetry (source gzmo_daemon)

Tier 5 — Edge parity
  cd ~/Projects/_foundation-audit/survey_edge-node/gzmo-daemon && npx tsc --noEmit

Deliverable — markdown report:
  | Tier | VERIFIED / FAILED / SKIPPED | Evidence (command output or file excerpt) |
  For each failure: root cause hypothesis + file:line if known.
  Do NOT modify vault.db or run dream consolidation.
```

### Policy-change test (after editing `[chaos]` or pulse logic)

Use the full regression prompt above, plus:

```
Before/after: record gzmo.toml [chaos] block.
Tier 2 is MANDATORY — append results to chaos-breathing-lab/RESULTS.md.
Compare max ρ vs previous matrix_summary.tsv baseline.
Re-run Tier 3 with both /story load AND /stabilize recovery path.
```

### TUI-only smoke (optional add-on)

```
After Tier 3, if time permits:
  unset CARGO_TARGET_DIR && ./target/release/gzmo   # TUI entry
  Verify /chaos and /stabilize appear in command palette and execute.
  (Use control-cli harness if available; otherwise document manual steps taken.)
```

### Tier 3 redo only (story-load gap closure)

```
Tier 3 redo — story load only. QA only, no code changes.

Prerequisites:
  pkill -f './target/release/gzmo daemon' || true
  export GZMO_LLM_URL=http://localhost:8000/v1/chat/completions
  export GZMO_LLM_MODEL=qwen3.6-35b-mtp
  bash scripts/qa/tier3_stress_chat.sh

Report table: Action | ρ mod | forcing/breath | story text excerpt (first line).
Pass: non-empty stories, ρ mod movement after crystallization, ρ < 7, stabilize drop ~1.0.
```

### What “extensive” does NOT mean

- Do not run DreamEngine, Spark cron, or vault promotion tests unless explicitly asked.
- Do not run daemon + chat concurrently on the same `data/`.
- Do not re-implement missing features — file gaps as test failures.

---

*Agent onboarding doc. Update when runtime wiring or default policy changes.*
