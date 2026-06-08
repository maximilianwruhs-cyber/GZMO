# Limit-Cycle Specs — Mathematical Map (Engineering Canon)

**Authority:** [`CHAOS_RHO_CONTROL_MODEL.md`](CHAOS_RHO_CONTROL_MODEL.md)  
**Handoff (step-by-step):** [`CHAOS_RHO_IMPLEMENTATION_HANDOFF.md`](CHAOS_RHO_IMPLEMENTATION_HANDOFF.md)

Distilled Rosetta for the **2026-06-08 limit-cycle design proposals** (mythological drafts removed from `gzmo-chaos/`). Use this for review, implementation, and lab design — not narrative lore.

---

## 0. Canonical model (our point of view)

`gzmo-chaos` is a **hybrid dynamical system**: a continuous **plant** driven by a discrete **accumulator** with saturation.

### 0.1 State partition

| Subsystem | State | Domain | Update rate |
|-----------|-------|--------|-------------|
| **Plant** \(\mathcal{P}\) | \((x,y,z) \in \mathbb{R}^3\) | Lorenz phase space | RK4 per tick, \(dt=0.005\) |
| **Accumulator** \(\mathcal{A}\) | \(\rho_{\mathrm{mod}} \in [-10,10]\) | Control offset | Discrete per PulseLoop tick |
| **Coupling** | \(\rho = 28 + \rho_{\mathrm{mod}}\) | Rayleigh number | Applied to plant each tick |
| **Output map** \(\mathcal{O}\) | \((T, N_{\mathrm{tok}}, v)\) | LLM params | Normalized clamps on \((x,y,z)\) |

Plant ODE (code: `chaos.rs`):

\[
\dot{x} = \sigma(y-x),\quad
\dot{y} = x(\rho - z) - y,\quad
\dot{z} = xy - \beta z,
\qquad \rho = 28 + \rho_{\mathrm{mod}}.
\]

**Critical distinction:** Specs that speak of a “limit cycle in phase space” mean a closed orbit in \((x,y,z)\). **We do not implement that.** Semantic events affect **\(\rho_{\mathrm{mod}\) only** — parameter feedback on an otherwise standard Lorenz plant.

### 0.2 Accumulator dynamics (shipped, discrete-time)

Per tick \(n\) (174 BPM \(\Rightarrow\) \(\Delta t_{\mathrm{tick}} \approx 0.345\,\mathrm{s}\)):

**Impulse (crystallization):**
\[
\rho_{\mathrm{mod}}[n^+] = \mathrm{clamp}\!\left(\rho_{\mathrm{mod}}[n^-] + \sum_i \Delta\rho_i,\; [-10,10]\right)
\]

**Dissipation (leaky integrator):**
\[
\rho_{\mathrm{mod}}[n+1] = \mathrm{clamp}\!\left((1-k)\,\rho_{\mathrm{mod}}[n^+],\; [-10,10]\right)
\]

**Stochastic forcing** \(\sum_i \Delta\rho_i\): table in `CHAOS_RHO_CONTROL_MODEL.md` §2 (joke \(-0.2\), story \(+0.5\), etc.).

**Equilibrium (mean-field):** If \(\mathbb{E}[\sum \Delta\rho] = \mu\) per tick and decay is the only restoration,
\[
\mu \approx k\,\rho_{\mathrm{mod}}^{\*}
\quad\Rightarrow\quad
\rho_{\mathrm{mod}}^{\*} \approx \mu/k.
\]
This is a **stable fixed point** of the discrete map (for \(k \in (0,1)\)), **not** a limit cycle.

### 0.3 Failure mode (what “saturation” means mathematically)

When \(\rho_{\mathrm{mod}} \to 10\):

1. \(\mathcal{O}(x,y,z)\) hits normalization clamps \(\Rightarrow\) **loss of sensitivity** in LLM outputs.
2. Lorenz \(\mathcal{P}\) remains numerically integrated; **no ODE blow-up**.
3. Agent “death” (if any) is **energy depletion** in `engine.rs` — orthogonal subsystem.

Specs conflate (2) and (3) under “suicide machine.” **Engineering:** open-loop accumulation \(\Rightarrow\) **actuator saturation** on \(\rho_{\mathrm{mod}}\).

### 0.4 Name collision warning

| Symbol in lore specs | Engineering meaning | Code symbol |
|----------------------|---------------------|-------------|
| `Phase::Inhale / Exhale` | \(\mathrm{sign}(\dot{\rho}_{\mathrm{mod}})\) or \(\mathrm{sign}(\Delta\rho_{\mathrm{mod}})\) | **Does not exist** — use `rho_forcing_sign` |
| `Phase::Idle / Build / Drop` | Hardware-tension regime | `chaos::Phase` in `engine.rs` |

Never overload `chaos::Phase` with Inhale/Exhale. Proposed type: `RhoForcingSign` or `RhoBreathPhase`.

---

## 1. Master equation lineage (all specs → one family)

All three documents describe variations of **one hybrid template**:

\[
\rho_{\mathrm{mod}}[n^+] = \mathrm{clamp}\!\left(\rho_{\mathrm{mod}}[n^-] + \underbrace{\sum_i \Delta\rho_i}_{\text{bursty forcing } u[n]},\; [-10,10]\right)
\]

\[
\rho_{\mathrm{mod}}[n+1] = \mathrm{clamp}\!\left(\rho_{\mathrm{mod}}[n^+] - \underbrace{\mathcal{R}(\rho_{\mathrm{mod}}[n^+])}_{\text{restoration}},\; [-10,10]\right)
\]

| Document | Restoration \(\mathcal{R}\) | Continuous-time metaphor |
|----------|---------------------------|--------------------------|
| **Blueprint Phase 1** (shipped) | \(k\,\rho_{\mathrm{mod}}\) i.e. \((1-k)\rho\) | \(\dot{\rho}_{\mathrm{mod}} = -k\rho_{\mathrm{mod}}\) |
| **SPEC V2** | \(\alpha\,(\rho_{\mathrm{mod}} - \rho_0)^n\) | Power-law restoring drift |
| **MASTER SPEC** | \(\alpha\,\tanh(\beta(\rho_{\mathrm{mod}} - \rho_0))\) | Bounded sigmoid restoring drift |
| **Lab `nonlinear_decay`** | \(k(1+\alpha|\rho_{\mathrm{mod}}|)\,\rho_{\mathrm{mod}}\) | State-dependent leak gain |
| **Control model §6** | \(k(1+\alpha|\rho_{\mathrm{mod}}|)\,\rho_{\mathrm{mod}}\) | Same as lab |

**Dimensional correction (applies to V2 and MASTER):** Lore specs set \(\rho_0 = 28\) (full Lorenz \(\rho\)). Code state is **offset** \(\rho_{\mathrm{mod}}\) with equilibrium near **\(0\)**, not \(28\). Correct center:

\[
\mathcal{R}(\rho_{\mathrm{mod}}) = \alpha\,|\rho_{\mathrm{mod}}|^n \operatorname{sign}(\rho_{\mathrm{mod}})
\quad\text{or}\quad
\alpha\,\tanh(\beta\,\rho_{\mathrm{mod}}).
\]

Using \(\rho_0=28\) in \((\rho_{\mathrm{mod}}-28)^n\) or \(\tanh(\beta(\rho_{\mathrm{mod}}-28))\) with \(\rho_{\mathrm{mod}}\in[-10,10]\) forces restoration **always toward large negative \(\rho_{\mathrm{mod}}\)** — not “return to baseline.”

---

## 2. Document-by-document map

### 2.1 Blueprint lineage (Phase 1–3 proposal)

| Lore section | Lore claim | Engineering translation | Status |
|--------------|------------|-------------------------|--------|
| Cosmological engine | Meaning → physical constants | **Semantic-to-parameter coupling:** crystallization \(\Rightarrow\) permanent \(\Delta\rho_{\mathrm{mod}}\) | Verified mechanism |
| Suicide loop | Lorenz collapses | **Open-loop accumulation:** \(\rho_{\mathrm{mod}}\uparrow\) with no dissipation \(\Rightarrow\) clamp \(\Rightarrow\) \(\mathcal{O}\) saturates | Verified (pre-port) |
| Limit cycle (Tao) | Closed orbit in time | **Misnomer:** target is **bounded \(\rho_{\mathrm{mod}}\)** under stochastic \(u[n]\); not a Poincaré cycle in \((x,y,z)\) | Reject as dynamical claim |
| Inhale | Stories expand attractor | **Positive impulse:** \(\Delta\rho > 0\) on story/persona crystallization | Shipped |
| Exhale | Nonlinear pull to baseline | **Dissipation:** originally unspecified; shipped as \((1-k)\rho_{\mathrm{mod}}\) | Phase 1 shipped |
| Phase 1: baseline pull | Constant decay | \(k=0.001\), `apply_rho_decay` | **Shipped** |
| Phase 2: nonlinear oscillation | Periodic limit cycle | **Proposed extension** — see §1 lineage; **not shipped**; lab did not show oscillation | Proposed |
| Phase 3: Synapse heartbeat | Observable Inhale/Exhale | **`rho_forcing_sign`**, EMA `rho_breath_phase`, Synapse `chaos.rho_telemetry` | **Shipped** |

**Blueprint continuous-time idealization (Phase 1 only):**

\[
\dot{\rho}_{\mathrm{mod}} = \mu(t) - k\,\rho_{\mathrm{mod}},
\quad \mu(t) = \sum_i \Delta\rho_i\,\delta(t - t_i).
\]

Steady mean \(\bar{\mu} \approx k\,\rho_{\mathrm{mod}}^{\*}\). **Classifier:** first-order leaky integrator with impulsive input — **not** a limit cycle.

---

### 2.2 V2 power-law proposal (lab-negative)

| Lore section | Lore equation / claim | Corrected engineering form | Dynamical class | Status |
|--------------|----------------------|---------------------------|-----------------|--------|
| §2 linear decay “single fixed point” | \((1-k)\rho\) | Correct for **stationary distribution** under noise; fixed point of **mean**, not “only one point” | Stable node (discrete) | Shipped |
| §2.1 Restoration | \(\alpha(\rho_{\mathrm{mod}}-\rho_0)^n\), \(\rho_0=28\) | \(\alpha\,|\rho_{\mathrm{mod}}|^n\operatorname{sign}(\rho_{\mathrm{mod}})\) | Nonlinear dissipative map | **Not shipped** |
| §2.1 \(n=1\) “damped harmonic oscillator” | — | **Incorrect analogy:** 1D dissipative drift without inertia is **not** a harmonic oscillator; need \(\ddot{\rho}\) term for that | — | Reject |
| §2.1 \(n>1\) “violent rapid return” | — | Stronger restoration at large \(|\rho_{\mathrm{mod}}|\) — matches lab `nonlinear_decay` intent | State-dependent leak | Lab tested, **lost to linear_fast** |
| §2.2 Inhale | \(\rho(t^+)=\rho(t^-)+\Delta\rho\) | Same as impulse step in §0.2 | Impulsive map | Shipped |
| §2.3 Limit cycle equilibrium | \(\int_0^T u\,dt = \int_0^T \mathcal{R}\,dt\) | **Period-averaged balance** — defines **steady oscillation** only if trajectory is periodic. With one-state dissipative map + noise → **almost surely aperiodic**; balance holds in **expectation**, not as a closed orbit | Equilibrium / stationary measure | Aspirational |
| Layer 1 Driver | `ChaosEvent::StoryGenerated` | Stochastic \(\Delta\rho\) at crystallization | Forcing \(u[n]\) | Shipped |
| Layer 2 Governor | nonlinear restore | \(\mathcal{R}(\rho_{\mathrm{mod}})\) per §1 | Restoration | Linear default; tanh opt-in |
| Layer 3 Observer | Synapse + TriggerEngine | **`rho_mod_delta`, `rho_effective`, `rho_forcing_sign`**, EMA breath, Synapse export | Observation \(y[n]\) | **Shipped** |
| Phase I | `rho -= alpha * rho.powf(n)` | Discrete: \(\rho \leftarrow \rho - \alpha|\rho|^n\operatorname{sign}(\rho)\) | Nonlinear map | **Rejected** (lab) |
| Phase II | \(\dot{\rho}_{\mathrm{mod}} \gtrless 0\) | \(\mathrm{sign}(\rho_{\mathrm{mod}}[n]-\rho_{\mathrm{mod}}[n-1])\) + EMA | Hysteresis-free classifier | **Shipped** |
| Phase III | Agent restorative events | **`/stabilize`**, \(\Delta\rho_{\mathrm{stab}}=-1\) | External control \(u_{\mathrm{ext}}\) | **Shipped** |

**Lab discretization of V2 intent** (`chaos-breathing-lab`, `PolicyKind::NonlinearDecay`):

\[
\rho_{\mathrm{mod}}[n+1] = (1 - k(1 + \alpha|\rho_{\mathrm{mod}}[n^+]|))\,\rho_{\mathrm{mod}}[n^+].
\]

`active_story_30s`: max \(\rho_{\mathrm{mod}} = 9.99\) (fail) vs `linear_decay_fast` max \(5.99\) (pass). **Conclusion:** V2 restoration **does not beat** shipped law under measured forcing.

---

### 2.3 MASTER tanh + EMA proposal

| Lore section | Lore equation / claim | Corrected engineering form | Notes | Status |
|--------------|----------------------|---------------------------|-------|--------|
| §1 Living organism | Bounded oscillation | **Homeostasis:** \(\rho_{\mathrm{mod}}\) confined; **oscillation optional** | Do not require periodic orbit for “alive” | Partial (homeostasis shipped) |
| §2.1 Governing ODE | \(\dot{\rho} = u - \alpha\tanh(\beta(\rho_{\mathrm{mod}}-\rho_0))\) | \(\dot{\rho}_{\mathrm{mod}} = u - \alpha\tanh(\beta\rho_{\mathrm{mod}})\) | Fix \(\rho_0\) → \(0\) | Proposed |
| §2.1 \(\tanh\) for RK4 stability | Bounded restoration | **Category error:** RK4 integrates **Lorenz** \((x,y,z)\); \(\rho_{\mathrm{mod}}\) is **not integrated by RK4** | Restoration stability is discrete-map issue | Reject rationale |
| §2.2 Inhale | Stochastic \(\Delta\rho\) | Same as \(u[n]\) | — | Shipped |
| Layer I Subconscious | Story \(\Rightarrow\) \(\Delta\rho\) | Forcing channel | — | Shipped |
| Layer II Homeostatic reflex | \(\tanh\) in `thoughts.rs` | \(\mathcal{R}=\alpha\tanh(\beta\rho_{\mathrm{mod}})\) per tick | Smooth saturation of restore rate | **Shipped opt-in** (`rho_restore_alpha`) |
| Layer III Consciousness | EMA of \(\dot{\rho}_{\mathrm{mod}}\) | \(v[n] = (1-\gamma)v[n-1] + \gamma\,\Delta\rho_{\mathrm{mod}}[n]\); phase \(=\mathrm{sign}(v)\) | Low-pass on increment | **Shipped** |
| Phase I Sigmoidal transition | Replace linear decay | Lab `tanh_decay` max ρ 0.93 vs linear_fast 5.99 | — | **Shipped opt-in** |
| Phase II Rhythmic pulse | `Phase::Inhale/Exhale` | `RhoBreathPhase` from EMA — **not** `chaos::Phase` | — | **Shipped** |
| Phase III `skill_stabilize` | Forced exhale | \(\rho_{\mathrm{mod}} \mathrel{+}= \Delta\rho_{\mathrm{stab}}\) or \(k_{\mathrm{boost}}\) timer | External feedback | **Shipped** |
| §5 Success criterion | **Stable periodic oscillation** | **Strict:** closed orbit in \(\rho_{\mathrm{mod}}\) alone requires **nonlinear + energy injection + phase memory** (e.g. relaxation oscillator). **Relaxed (operational):** \(\mathbb{P}(\rho_{\mathrm{mod}} \geq 10) \approx 0\) under story load | See §4 | Relaxed **met**; strict **not met** |

**Discrete MASTER Phase I candidate:**

\[
\rho_{\mathrm{mod}}[n+1] = \mathrm{clamp}\!\left(
\rho_{\mathrm{mod}}[n^+] - \alpha\,\tanh(\beta\,\rho_{\mathrm{mod}}[n^+]),
\; [-10,10]\right).
\]

**Discrete MASTER Phase II (observer):**

\[
v[n] = (1-\gamma)\,v[n-1] + \gamma\,\bigl(\rho_{\mathrm{mod}}[n] - \rho_{\mathrm{mod}}[n-1]\bigr),
\qquad
\text{breath\_phase}[n] = \mathrm{sign}(v[n]) \in \{-1,0,+1\}.
\]

Shipped shortcut: `rho_forcing_sign[n] = \mathrm{sign}(\rho_{\mathrm{mod}}[n]-\rho_{\mathrm{mod}}[n-1])` (no EMA, \(\gamma=1\)).

**Discrete MASTER Phase III:**

\[
\rho_{\mathrm{mod}} \leftarrow \rho_{\mathrm{mod}} + \Delta\rho_{\mathrm{stab}},
\quad \Delta\rho_{\mathrm{stab}} < 0 \text{ (e.g. } -1.0\text{)},
\quad \text{or } k \leftarrow k_{\mathrm{boost}} \text{ for } M \text{ ticks}.
\]

---

## 3. Unified layer map (all three specs)

| Lore layer | Mathematical role | Input | Output | Implementation |
|------------|-------------------|-------|--------|----------------|
| **Driver / Subconscious** | Forcing \(u[n]\) | Crystallization events | \(\sum \Delta\rho_i\) | `thoughts.rs::crystallize` |
| **Governor / Homeostatic reflex** | Restoration \(\mathcal{R}\) | \(\rho_{\mathrm{mod}}[n^+]\) | Dissipated state | `apply_rho_restoration` (linear default; tanh opt-in) |
| **Plant** | Lorenz \(\mathcal{P}\) | \(\rho, \sigma\) | \((x,y,z)\) | `chaos.rs::LorenzAttractor` |
| **Output map** | \(\mathcal{O}\) | \((x,y,z)\) | LLM params | `pulse.rs` Lorenz mappers |
| **Observer / Consciousness** | \(y[n] = h(\rho_{\mathrm{mod}}[n])\)) | \(\rho_{\mathrm{mod}}\) history | Telemetry | `ChaosSnapshot` fields |
| **Actuator (agent)** | External \(u_{\mathrm{ext}}\) | Agent command | \(\Delta\rho_{\mathrm{stab}}\) | `/stabilize`, `skill_stabilize.sh` **shipped** |
| **Event bus (Synapse)** | Publish \(y[n]\) | Snapshot | IPC | `chaos.rho_telemetry` **shipped** (daemon) |

**Separate subsystem (not in ρ specs):** `engine.rs` energy ODE / phase machine (`Idle/Build/Drop`) — hardware tension, **not** \(\rho_{\mathrm{mod}}\) breathing.

---

## 4. What “limit cycle” means in each document vs engineering

| Claim | Lore meaning | Mathematical reality in our stack |
|-------|--------------|-----------------------------------|
| Limit cycle in **phase space** | Closed orbit in \((x,y,z)\) | **False target.** Lorenz at fixed \(\rho\) is chaotic; semantic events do not sculpt a new attractor topology in 3D. |
| Limit cycle in **\(\rho_{\mathrm{mod}}\)** | Periodic breathing | Requires **relaxation oscillator** or **2-state hysteresis** (control model §6). Impulse + linear leak \(\Rightarrow\) **fixed point + noise**, not guaranteed periodicity. |
| “Engine must dance” | Visible oscillation | **Operational substitute:** \(\rho_{\mathrm{mod}}\) varies with bounded variance, \(\max \rho_{\mathrm{mod}} < 10\) under story load — **achieved** (lab + live). |
| Steady state Phase 1 | \(\dot{\rho}_{\mathrm{mod}}=0\) on average | \(\mathbb{E}[u] \approx k\rho_{\mathrm{mod}}^{\*}\) | **Achieved** |
| SPEC V2 integral balance | Periodic \(T\) exists | True **iff** trajectory is periodic; **not proven** for proposed nonlinear \(\mathcal{R}\) |

**Van der Pol–style relaxation (if strict oscillation required):**

\[
\rho_{\mathrm{mod}}[n+1] = \rho_{\mathrm{mod}}[n] + \Delta t \cdot \bigl(\mu(\rho_{\mathrm{mod}}[n]) + u[n]\bigr),
\quad
\mu(\rho) = \rho - \rho^3/3 \quad \text{(or equivalent hysteresis)}.
\]

None of the three lore specs write this; they only strengthen \(\mathcal{R}\), which **stabilizes** rather than **oscillates**.

---

## 5. Implementation & validation matrix

| Math object | Blueprint | SPEC V2 | MASTER | Code | Lab |
|-------------|-----------|---------|--------|------|-----|
| Impulse table \(\Delta\rho_i\) | ✓ | ✓ | ✓ | `thoughts.rs` | ✓ |
| Linear leak \(k\) | Phase 1 | baseline | superseded | `k=0.001` | `linear_decay_fast` **winner** |
| Power-law \(\mathcal{R}\) | Phase 2 | Phase I | — | — | `nonlinear_decay` **fail** active_story |
| Tanh \(\mathcal{R}\) | — | — | Phase I | `rho_restore_alpha` opt-in | `tanh_decay` **pass** |
| `rho_forcing_sign` | — | Phase II | Phase II | `pulse.rs` | trend column in CSV |
| EMA phase | — | — | Layer III | `rho_breath_phase` | shipped |
| `skill_stabilize` | — | Phase III | Phase III | `feedback.rs`, CLI | shipped |
| Synapse export | Phase 3 | Layer 3 | — | `chaos_bootstrap.rs` | shipped |

---

## 6. Recommended reading order

1. **`CHAOS_RHO_CONTROL_MODEL.md`** — shipped law (start here)  
2. **This map** — proposal lineage, lab verdicts, equation corrections  
3. **`CHAOS_RHO_IMPLEMENTATION_HANDOFF.md`** — verify tiers, daemon, workstreams  
4. **`chaos-breathing-lab/RESULTS.md`** — simulation numbers

---

## 7. One-line summary per proposal lineage

| Lineage | Engineering one-liner |
|---------|----------------------|
| **Blueprint** | Leaky integrator + impulses fixes saturation; telemetry + EMA shipped; strict limit cycle **rejected** as dynamical target. |
| **V2 power-law** | \(\mathcal{R}(|\rho_{\mathrm{mod}}|^n)\) — **lab-negative** vs `linear_decay_fast`; do not implement. |
| **MASTER tanh** | Bounded \(\tanh\) restore + EMA + stabilize — **lab-validated**, shipped **opt-in**; strict periodic oscillation **not required**. |

---

*Canonical equation Rosetta for gzmo-chaos ρ proposals. Update when policies or observers change.*
