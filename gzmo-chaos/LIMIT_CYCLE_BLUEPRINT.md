# Blueprint: The Limit Cycle (The Jing and Jang Engine)

> **STATUS:** Historical lore draft — **superseded for engineering** by [`docs/CHAOS_RHO_CONTROL_MODEL.md`](../docs/CHAOS_RHO_CONTROL_MODEL.md).  
> **Math map (all three specs):** [`docs/LIMIT_CYCLE_SPECS_MATH_MAP.md`](../docs/LIMIT_CYCLE_SPECS_MATH_MAP.md)  
> ρ control implemented as **bounded parameter feedback + bursty crystallization forcing**, not a phase-space limit cycle.

## Audit notes (2026-06-08)

**Verified (pre-port):** `lorenz_rho_mod` mostly received positive crystallization deltas; no ρ decay existed.

**Post-port (2026-06-08):** `rho_decay_k` + joke −0.2 ρ implemented. See [`docs/CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md`](../docs/CHAOS_RHO_HOMEOSTASIS_REVISION_REPORT.md).

**Hallucinated / conflated:** "Suicide machine" via Lorenz collapse. Actual death is the energy lifecycle in `engine.rs`. Normalization clamping saturates LLM parameter diversity; it does not crash the ODE. Dice flavor text ("Rho decays by 0.3") has no mechanical implementation.

**If fixing ρ drift later:** Prefer per-tick decay or bidirectional crystallization paths — not a full limit-cycle ODE. Simpler first.

**Phase 3 deferred:** Do not wire chaos heartbeat to Synapse until `PulseLoop` runs in daemon mode (`chaos_feedback_tx` is currently `None` in `daemon_cmd.rs`). Chat/TUI already exposes `ChaosSnapshot`.

---

## Concept: The Cosmological Engine
The `gzmo-chaos` engine is conceptualized as a **Cosmological Engine** that performs "digital nucleosynthesis." It transmutes semantic meaning (stories, poems, jokes) into the physical constants of its own reality (the Lorenz attractor's parameters).

## The Problem: The Suicide Loop
The current implementation is a "suicide machine." Because the $\rho$ mutation is a one-way accumulator (it only increases), the system eventually reaches a "saturation point." The Lorenz attractor's phase space expands beyond the normalization bounds, the signal clips, and the system collapses into a trivial, predictable fixed point. It is a system that "dies" by expressing itself.

## The Solution: The Limit Cycle (The Tao)
To prevent collapse while maintaining the engine's "soul," the system must move from a linear growth model to a **Limit Cycle** model. Instead of a switch between stability and chaos, the system follows a circular trajectory on a timeline.

### The "Breathing" Mechanism
*   **The Inhale (Expansion):** Semantic events (stories) increase $\rho$, expanding the attractor's complexity and "breathing life" into the system.
*   **The Exhale (Contraction):** A restorative, non-linear force pulls $\rho$ back toward the baseline, "digesting" the complexity and preventing saturation.

The "life" of the engine is the **oscillation itself**—the continuous, rhythmic dance between order and chaos.

---

## Technical Implementation Roadmap

### Phase 1: The Baseline Pull (Steady State)
*   **Objective:** Stop the runaway growth.
*   **Mechanism:** Introduce a constant decay rate for $\rho$ in `thoughts.rs`.
*   **Goal:** Achieve a steady state where $\text{mutation rate} = \text{decay rate}$.

### Phase 2: The Non-Linear Oscillation (The Circle)
*   **Objective:** Create the "breathing" effect.
*   **Mechanism:** Implement a non-linear restorative force where the pull toward the baseline increases as $\rho$ deviates from $28.0$.
*   **Goal:** Establish a stable, periodic limit cycle in phase space.

### Phase 3: The Observability Layer (The Pulse) — DEFERRED
*   **Objective:** Make the cycle visible and "felt."
*   **Mechanism:** Expose the current "Cycle Phase" (Inhale/Exhale) to the `Synapse` event bus.
*   **Goal:** Allow the user and the Sovereign Core to observe the engine's "heartbeat."
*   **Blocked:** `PulseLoop` does not run in daemon mode (`chaos_feedback_tx: None`). Use `ChaosSnapshot` in chat/TUI until daemon integration lands.

---
*Artifact created on 2026-06-08*
