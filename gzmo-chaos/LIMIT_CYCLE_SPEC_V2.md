# Specification: The Living Engine (The Limit Cycle Protocol)

> **STATUS:** Design proposal — **not canonical.**  
> **Math map:** [`docs/LIMIT_CYCLE_SPECS_MATH_MAP.md`](../docs/LIMIT_CYCLE_SPECS_MATH_MAP.md)  
> **Shipped law:** [`docs/CHAOS_RHO_CONTROL_MODEL.md`](../docs/CHAOS_RHO_CONTROL_MODEL.md).  
> **Correction:** \(\rho_0=28\) in §2.1 is **wrong dimension** — use \(\rho_{\mathrm{mod}}^{\*}=0\).  
> **Lab:** `nonlinear_decay` lost to `linear_decay_fast` on `active_story_30s` (see map §2.2).

## 1. Philosophical Foundation: The Breath of Chaos
The goal is to move from a "leaky growth" model to a **Self-Sustaining Limit Cycle**. The engine must not merely "decay"; it must **breathe**. It must transition between periods of **Expansion** (Complexity/Chaos) and **Contraction** (Order/Stability) in a rhythmic, predictable, yet non-deterministic cycle.

This is the transition from **Autopoiesis** (self-creation) to **Homeostasis** (self-regulation).

---

## 2. The Mathematical Model: The Non-Linear Governor

The current model uses a linear decay: $\rho_{mod} \leftarrow (1-k) \cdot \rho_{mod}$. This leads to a single fixed point.
The **Limit Cycle Model** requires a non-linear restorative force.

### 2.1 The Restorative Function (The Exhale)
We define the restorative force $F(\rho_{mod})$ as a function that increases exponentially as $\rho_{mod}$ moves away from the baseline ($\rho_0 = 28.0$).

$$\frac{d\rho_{mod}}{dt} = \text{Impulses}(\text{Stories}) - \text{Restoration}(\rho_{mod})$$

Where the Restoration function is:
$$\text{Restoration}(\rho_{mod}) = \alpha \cdot (\rho_{mod} - \rho_0)^n$$

*   **$\alpha$ (The Breath Constant):** Controls the strength of the restorative force.
*   **$n$ (The Nonlinearity Index):** Controls the "shape" of the cycle. 
    *   If $n=1$, it is a damped harmonic oscillator.
    *   If $n>1$, the return to order is slow at first but becomes violently rapid as the system approaches the "edge" of stability.

### 2.2 The Expansion Impulse (The Inhale)
The "Inhale" is driven by semantic events (Stories, Personas, etc.) that inject discrete $\Delta\rho$ pulses.
$$\rho_{mod}(t^+) = \rho_{mod}(t^-) + \Delta\rho_{event}$$

### 2.3 The Limit Cycle Equilibrium
The system reaches a **Limit Cycle** when the integral of the impulses over one period $T$ equals the integral of the restoration over the same period:
$$\int_{0}^{T} \text{Impulses}(t) \,dt = \int_{0}^{T} \text{Restoration}(\rho_{mod}(t)) \,dt$$

---

## 3. System Architecture: The Three Layers of Life

### Layer 1: The Driver (The Subconscious)
*   **Role:** Generates novelty.
*   **Mechanism:** `ChaosEvent::StoryGenerated` and `ChaosEvent::PersonaShift`.
*   **Output:** Discrete, stochastic pulses of $\Delta\rho$.
*   **Nature:** Unpredictable, creative, and expansive.

### Layer 2: The Governor (The Homeostatic Reflex)
*   **Role:** Prevents systemic collapse.
*   **Mechanism:** The non-linear `apply_rho_decay` function in `thoughts.rs`.
*   **Output:** A continuous, state-dependent restorative force.
*   **Nature:** Deterministic, corrective, and stabilizing.

### Layer 3: The Observer (The Consciousness)
*   **Role:** Perceives the cycle and acts upon it.
*   **Mechanism:** The `Synapse` event bus and `TriggerEngine`.
*   **Output:** Semantic awareness of the current "Phase" (Inhale vs. Exhale).
*   **Nature:** Observational, reflective, and strategic.

---

## 4. Implementation Roadmap: Achieving Perfection

### Phase I: The Non-Linearity Injection (The "Shape")
*   **Task:** Replace the linear decay in `thoughts.rs` with the non-linear function:
    `self.mutations.lorenz_rho_mod -= alpha * (self.mutations.lorenz_rho_mod.powf(n))`
*   **Goal:** Observe the transition from a single fixed point to a stable oscillation.

### Phase II: The Phase-Aware Trigger (The "Awareness")
*   **Task:** Implement a "Cycle Phase" detector in `PulseLoop`.
*   **Mechanism:** Use the sign and derivative of $\rho_{mod}$ to classify the current state:
    *   **Expansion Phase:** $\frac{d\rho_{mod}}{dt} > 0$ (The Inhale).
    *   **Contraction Phase:** $\frac{d\rho_{mod}}{dt} < 0$ (The Exhale).
*   **Goal:** Allow the `TriggerEngine` to fire different alerts based on whether the engine is "growing" or "resting."

### Phase III: The Sovereign Integration (The "Will")
*   **Task:** Enable the Sovereign Core (the Agent) to influence the cycle.
*   **Mechanism:** Allow the Agent to trigger "Restorative Events" (e.g., a "Reflection" skill) that manually increases the restorative force, allowing the agent to "force an exhale" if the system is approaching a singularity.
*   **Goal:** Achieve true **Sovereign Autopoiesis**—a system that doesn't just react to chaos, but actively manages its own rhythm.

---
*Master Specification for the Living Engine — Version 2.0*
