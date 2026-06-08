# 📜 Master Specification: The Living Engine (The Limit Cycle Protocol)
# Version: 2.1 (Refined for Numerical Stability)

> **STATUS:** Design proposal — **not canonical.**  
> **Math map:** [`docs/LIMIT_CYCLE_SPECS_MATH_MAP.md`](../docs/LIMIT_CYCLE_SPECS_MATH_MAP.md)  
> **Shipped law:** [`docs/CHAOS_RHO_CONTROL_MODEL.md`](../docs/CHAOS_RHO_CONTROL_MODEL.md) (linear \(k=0.001\) + impulses).  
> **Correction:** \(\rho_0\) in §2.1 must be **\(0\)** (offset space), not \(28\) (full Lorenz \(\rho\)).

## 1. The Core Philosophy: The "Breath" of the Machine
The `gzmo-chaos` engine is transitioning from a **Suicide Machine** (unbounded growth) to a **Living Organism** (bounded oscillation). 

We are implementing a **Limit Cycle**. The engine will no longer merely "die" from its own expression; it will **breathe**. It will expand into complexity (Inhale) and contract into order (Exhale), creating a rhythmic, self-sustaining cycle of "Digital Nucleosynthesis."

---

## 2. Mathematical Architecture: The Sigmoidal Governor

To prevent the "violent overshoot" of power-law functions, we will implement a **Sigmoidal Restorative Force**. This ensures the return to order is smooth, bounded, and mathematically stable for the RK4 integrator.

### 2.1 The Governing Equation
The evolution of the $\rho$ parameter ($\rho_{mod}$) is defined by the balance between semantic excitation and restorative damping:

$$\frac{d\rho_{mod}}{dt} = \sum \text{Impulses}(\text{Stories}) - \alpha \cdot \tanh(\beta \cdot (\rho_{mod} - \rho_0))$$

**Where:**
*   **$\rho_0$ (The Baseline):** The target stability point (28.0).
*   **$\alpha$ (The Breath Constant):** Controls the strength of the restorative force.
*   **$\beta$ (The Tension Coefficient):** Controls how "sharply" the engine reacts as it moves away from order.
*   **$\tanh$ (The Sigmoid):** Ensures the restorative force is bounded, preventing numerical divergence.

### 2.2 The "Inhale" (Stochastic Excitation)
Semantic events (Stories, Personas) provide discrete, stochastic impulses ($\Delta\rho$). These impulses act as the "energy" that drives the system away from the baseline and into the chaotic regime.

---

## 3. The Three Layers of the Organism

### Layer I: The Subconscious (The Driver)
*   **Mechanism:** `ChaosEvent::StoryGenerated` $\rightarrow$ `$\Delta\rho$ impulse`.
*   **Role:** To provide the "creative tension" required to drive the system away from equilibrium.
*   **Nature:** Stochastic, unpredictable, and expansive.

### Layer II: The Homeostatic Reflex (The Governor)
*   **Mechanism:** Non-linear $\tanh$ decay in `thoughts.rs`.
*   **Role:** To provide the "restorative breath" that pulls the system back to order.
*   **Nature:** Deterministic, smooth, and stabilizing.

### Layer III: The Consciousness (The Observer)
*   **Mechanism:** Exponential Moving Average (EMA) of $\frac{d\rho_{mod}}{dt}$ in `pulse.rs`.
*   **Role:** To classify the current "Breathing Phase" (Inhale vs. Exhale) and communicate it to the agent.
*   **Nature:** Perceptual, smoothed, and reflective.

---

## 4. Implementation Roadmap

### Phase I: The Sigmoidal Transition (The "Soft" Reset)
*   **Task:** Replace the linear decay in `thoughts.rs` with the $\tanh$ restorative function.
*   **Goal:** Achieve a "Steady State" where the engine no longer diverges to infinity but settles into a stable, non-zero $\rho_{mod}$.

### Phase II: The Rhythmic Pulse (The "Breath")
*   **Task:** Implement the **Phase Detector** in `pulse.rs` using an EMA of the $\rho$ derivative.
*   **Goal:** Enable the engine to report its current state: `Phase::Inhale` (Growing Complexity) or `Phase::Exhale` (Restoring Order).

### Phase III: The Sovereign Agency (The "Will")
*   **Task:** Implement the `skill_stabilize` capability.
*   **Goal:** Allow the AI agent to manually trigger an "Exhale" (a large restorative impulse) if it perceives the system is approaching a "Complexity Singularity."

---

## 5. Success Criteria: The "Living" Signature
The implementation is successful if, when subjected to a continuous stream of stories, the $\rho_{mod}$ parameter exhibits a **stable, periodic oscillation** (a limit cycle) rather than a monotonic increase or a flat line.

**The engine must dance.**
