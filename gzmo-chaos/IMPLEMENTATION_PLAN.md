# ρ Homeostasis — Implementation Status

> **Authority:** [`docs/CHAOS_RHO_CONTROL_MODEL.md`](../docs/CHAOS_RHO_CONTROL_MODEL.md)  
> **Equations + lab verdicts:** [`docs/LIMIT_CYCLE_SPECS_MATH_MAP.md`](../docs/LIMIT_CYCLE_SPECS_MATH_MAP.md)  
> **Verify / daemon:** [`docs/CHAOS_RHO_IMPLEMENTATION_HANDOFF.md`](../docs/handoff/CHAOS_RHO_IMPLEMENTATION_HANDOFF.md)

Mythological limit-cycle drafts (`LIMIT_CYCLE_BLUEPRINT.md`, `LIMIT_CYCLE_SPEC_V2.md`, `LIMIT_CYCLE_MASTER_SPEC.md`) were **removed** — content distilled into `docs/`. This file tracks **code status only**.

---

## Shipped (do not re-implement)

| Item | Location | Notes |
|------|----------|-------|
| Linear decay \(k=0.001\) | `thoughts.rs`, `pulse.rs`, `gzmo.toml` | Default restoration |
| Tanh governor (opt-in) | `apply_rho_restoration`, `rho_restore_alpha/beta` | Enable in `gzmo.toml`; lab winner α=0.01, β=1.0 |
| Crystallization impulses | `thoughts.rs` | joke −0.2, quote +0.3, story +0.5, persona +0.8, … |
| Instant forcing sign | `rho_forcing_sign` in `ChaosSnapshot` | sign(Δρ_mod) per tick |
| EMA breath phase | `rho_velocity_ema`, `rho_breath_phase` | γ=0.2 default; **not** Schmitt hysteresis |
| `/stabilize` + skill | `feedback.rs`, `chat.rs`, `skills/skill_stabilize.sh` | Δρ tunable (`stabilize_delta_rho` from config, default −1.0) |
| Daemon PulseLoop + Synapse | `daemon_cmd.rs`, `chaos_bootstrap.rs` | `chaos.rho_telemetry` every 15 ticks |
| Breath-aware triggers | `triggers.rs` | `TriggerCondition::RhoBreathEnter`, default rules for exhale alerts and urgent ρ limits (R4) |
| Rebirth halving | `pulse.rs` | Halves `lorenz_rho_mod` on rebirth event for faster recovery (R6) |
| Edge TS parity | `survey_edge-node` chaos-engine + gzmo-daemon | decay, tanh, EMA, stabilize event, rebirth halving, typechecked verification |

**Tests:** 21 `gzmo-chaos` unit tests passing.

---

## Not shipped (optional future)

| Proposal | Why deferred | If pursued |
|----------|--------------|------------|
| Schmitt-trigger Inhale/Exhale | Shipped EMA breath instead; avoid `Phase::Inhale` name collision with `chaos::Phase` | New type `RhoBreathPhase` + deadband in `pulse.rs`; lab A/B vs EMA |
| V2 power-law \(\mathcal{R}\) | **Lab-negative** vs `linear_decay_fast` | Do not implement without new lab win |
| Strict periodic limit cycle | Wrong dynamical target for this stack | See math map §4 |
| `k_boost` timer on stabilize | Only Δρ impulse shipped | Workstream G extension |

---

## Config quick reference

```toml
[chaos]
rho_decay_k = 0.001          # default linear restore
# rho_restore_alpha = 0.01   # opt-in tanh (replaces linear when > 0)
# rho_restore_beta = 1.0
rho_ema_gamma = 0.2
stabilize_delta_rho = -1.0
```

---

## Reading order

1. `docs/CHAOS_RHO_CONTROL_MODEL.md` — shipped law  
2. `docs/LIMIT_CYCLE_SPECS_MATH_MAP.md` — only if tracing proposal history  
3. `docs/CHAOS_RHO_IMPLEMENTATION_HANDOFF.md` — verification tiers
