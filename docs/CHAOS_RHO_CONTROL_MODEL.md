# Chaos Engine — ρ Control Model (Engineering Spec)

**Replaces mythological framing** in [`LIMIT_CYCLE_BLUEPRINT.md`](../gzmo-chaos/LIMIT_CYCLE_BLUEPRINT.md) for implementation and review.  
**Revision history:** [`CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md`](CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md)  
**Implementation handoff:** [`CHAOS_RHO_IMPLEMENTATION_HANDOFF.md`](CHAOS_RHO_IMPLEMENTATION_HANDOFF.md)

---

## 1. What this system actually is

`gzmo-chaos` couples two layers:

1. **Plant (continuous):** Lorenz ODE integrated at 174 BPM; control parameter  
   \(\rho = 28.0 + \rho_{\mathrm{mod}}\).
2. **Accumulator (discrete):** `lorenz_rho_mod` updated by Thought Cabinet crystallization events (impulses) and per-tick decay (dissipation).

Semantic content does not create a limit cycle in \((x,y,z)\). It creates **bounded parameter feedback with bursty semantic forcing** on \(\rho_{\mathrm{mod}}\) — the same class of problem as Lorenz Rayleigh-number (\(\rho\)) control in classical and ML literature (parameter feedback / backstepping / restorative perturbations).

---

## 2. State and dynamics

### State variables

| Symbol | Code | Meaning |
|--------|------|---------|
| \(\rho_{\mathrm{mod}}\) | `mutations.lorenz_rho_mod` | Additive offset to Lorenz \(\rho\); clamped to \([-10, 10]\) |
| \(\rho\) | `lorenz.rho` | \(28.0 + \rho_{\mathrm{mod}}\) |
| \(k\) | `config.rho_decay_k` | Per-tick multiplicative decay gain |

### Discrete-time update (per PulseLoop tick)

After crystallization events on tick \(n\):

\[
\rho_{\mathrm{mod}}[n^+] = \mathrm{clamp}\Big(\rho_{\mathrm{mod}}[n^-] + \sum_i \Delta\rho_i,\; [-10, 10]\Big)
\]

\[
\rho_{\mathrm{mod}}[n+1] = \mathrm{clamp}\big((1 - k)\,\rho_{\mathrm{mod}}[n^+],\; [-10, 10]\big)
\]

Crystallization impulses \(\Delta\rho_i\) (examples):

| Event category | \(\Delta\rho\) |
|----------------|----------------|
| joke | \(-0.2\) |
| quote | \(+0.3\) |
| poem | \(+0.1\) |
| story | \(+0.5\) |
| persona | \(+0.8\) |

### Steady state (design intent)

Under mean impulse rate \(\mathbb{E}[\sum \Delta\rho]\) per tick, equilibrium approximately satisfies:

\[
\mathbb{E}\!\left[\sum \Delta\rho\right] \approx k \cdot \rho_{\mathrm{mod}}^{\*}
\]

Half-life of decay-only response: \(t_{1/2} \approx \ln(2)/k\) ticks (\(k{=}0.001 \Rightarrow\) ~693 ticks ≈ 4 min at 174 BPM).

### Coupled output

Lorenz coordinates map to LLM parameters via normalized clamps on \(x,y,z\). **Failure mode** when \(\rho_{\mathrm{mod}} \to 10\): normalization saturates → temperature/valence lose dynamic range (not ODE divergence).

---

## 3. Terminology map (retire mythological terms)

Use **engineering column** in code comments, docs, and PRs.

| Avoid (myth / blueprint) | Use instead (engineering) |
|--------------------------|---------------------------|
| Limit cycle in phase space | Bounded \(\rho_{\mathrm{mod}}\) under impulse + decay; Lorenz remains chaotic in operating band |
| Breathing | Parameter relaxation with bursty crystallization forcing |
| Inhale | Positive \(\Delta\rho\) impulse (crystallization) |
| Exhale | Multiplicative decay \((1-k)\rho_{\mathrm{mod}}\) |
| Tao / Jing-Jang | *(omit)* |
| Cosmological engine | Semantic-to-parameter coupling loop |
| Digital nucleosynthesis | Crystallization → permanent control-parameter mutation |
| Suicide machine | Open-loop accumulation → saturation / output clipping |
| Soul of the engine | Dynamic range of derived LLM control outputs |
| Heartbeat / Pulse (metaphor) | `PulseLoop` tick scheduler (174 BPM) |
| Cycle phase (Inhale/Exhale) | `rho_forcing_sign` (instant) or `rho_breath_phase` (EMA-smoothed) |

---

## 4. Implemented control law (2026-06-08, extended 2026-06-09)

| Component | Location | Value |
|-----------|----------|-------|
| Decay gain \(k\) | `pulse.rs`, `gzmo.toml` `[chaos].rho_decay_k` | `0.001` (set `0.0` to disable) |
| Tanh restore \(\mathcal{R}\) | `thoughts.rs`, `[chaos].rho_restore_alpha/beta` | opt-in (`alpha=0` default); lab winner α=0.01, β=1.0 |
| EMA gain \(\gamma\) | `pulse.rs`, `[chaos].rho_ema_gamma` | `0.2` |
| Joke impulse | `thoughts.rs` crystallize | \(\Delta\rho = -0.2\) |
| Manual stabilize | `ChaosEvent::Stabilize`, `/stabilize` | \(\Delta\rho = -1.0\) default |
| Clamp | `thoughts.rs` | \(\rho_{\mathrm{mod}} \in [-10, 10]\) |
| Daemon `PulseLoop` | `daemon_cmd.rs` + `chaos_bootstrap.rs` | Shipped |
| Shared bridge | `chaos_bootstrap.rs` | chat, TUI, daemon |

**Validation:** 18+ `gzmo-chaos` unit tests, lab sim, live CLI `/chaos` + `/stabilize`, daemon Tier 4.

**Not implemented:** state-dependent linear leak \(k(1+\alpha|\rho_{\mathrm{mod}}|)\) (lab-negative vs `linear_decay_fast`).

---

## 5. Observability (engineering roadmap)

| Metric | Definition | Status |
|--------|------------|--------|
| `rho_mod` | Current accumulator | In `ChaosSnapshot.mutations` |
| `rho_effective` | \(28 + \rho_{\mathrm{mod}}\) | Derivable |
| `rho_mod_delta` | \(\rho_{\mathrm{mod}}[n] - \rho_{\mathrm{mod}}[n-1]\) | **In `ChaosSnapshot`** |
| `rho_effective` | \(28 + \rho_{\mathrm{mod}}\) | **In `ChaosSnapshot`** |
| `rho_forcing_sign` | \(\mathrm{sign}(\rho_{\mathrm{mod\_delta}})\) ∈ \(\{-1,0,+1\}\) | **In `ChaosSnapshot`** |
| `rho_velocity_ema` | \((1-\gamma)v + \gamma\,\Delta\rho_{\mathrm{mod}}\) | **In `ChaosSnapshot`** |
| `rho_breath_phase` | \(\mathrm{sign}(v)\) ∈ \(\{-1,0,+1\}\) | **In `ChaosSnapshot`** |

Synapse `chaos.rho_telemetry` events append every 15 ticks in **daemon** mode (`SenseChaosRho` → `data/Synapse/events.jsonl`). Chat passes `synapse: None`.

---

## 6. Extensions (math-first, not myth-first)

| Goal | Control-theoretic form |
|------|------------------------|
| Slower mood, same stability | Reduce \(k\) or add negative \(\Delta\rho\) on more categories |
| Faster recovery after bursts | Increase \(k\) or rebirth scaling \(\rho_{\mathrm{mod}} \leftarrow \alpha \rho_{\mathrm{mod}}\) on energy rebirth |
| Stronger restoration far from baseline | \( \rho_{\mathrm{mod}} \leftarrow (1 - k(1 + \alpha|\rho_{\mathrm{mod}}|))\rho_{\mathrm{mod}} \) |
| True oscillation | Two-state or hysteresis scheduler on \(\rho_{\mathrm{mod}}\) (relaxation oscillator) — only if product needs periodic rhythm |

---

## 7. Relation to limit-cycle lore specs

| File | Role |
|------|------|
| [`LIMIT_CYCLE_BLUEPRINT.md`](../gzmo-chaos/LIMIT_CYCLE_BLUEPRINT.md) | Historical; Phase 1 shipped |
| [`LIMIT_CYCLE_SPEC_V2.md`](../gzmo-chaos/LIMIT_CYCLE_SPEC_V2.md) | Proposed power-law \(\mathcal{R}\) — lab-negative |
| [`LIMIT_CYCLE_MASTER_SPEC.md`](../gzmo-chaos/LIMIT_CYCLE_MASTER_SPEC.md) | Tanh \(\mathcal{R}\) + EMA — tanh **lab-validated**, shipped opt-in |
| **[`LIMIT_CYCLE_SPECS_MATH_MAP.md`](LIMIT_CYCLE_SPECS_MATH_MAP.md)** | **Rosetta:** lore → engineering equations |

Keep lore files as **design history**. Cite **this document** for shipped ρ behavior; cite **math map** when translating V2/MASTER proposals.

---

## 8. Deployment stance — Path A (chosen)

**Decision:** Keep **in-loop chaos** — `PulseLoop` drives Lorenz state and derived LLM parameters in chat/TUI. Scheduler-only spontaneity ([`gzmo-rebuild`](../../../gzmo-rebuild/README.md) Path B) is **not** the primary architecture.

| Path A commitment | Status |
|-------------------|--------|
| ρ leaky integrator + crystallization impulses | Shipped |
| Engineering terminology (`CHAOS_RHO_CONTROL_MODEL.md`) | Canonical |
| `rho_mod_delta` / `rho_effective` / `rho_forcing_sign` in snapshot | Shipped |
| EMA `rho_breath_phase` + `/stabilize` | Shipped |
| `chaos_bootstrap` (chat + TUI + daemon) | Shipped |
| Daemon `PulseLoop` | Shipped |
| Synapse ρ telemetry | Shipped (daemon only) |
| Tanh governor | Shipped opt-in (`rho_restore_alpha > 0`) |
| `edge-node` TS parity | Shipped (decay + tanh + EMA + Stabilize) |

**Next work on Path A:** optional enable tanh in production `gzmo.toml`; \(k\) tuning sweep if slower mood needed.

---

*Canonical engineering reference for gzmo-chaos ρ control.*
