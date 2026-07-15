# Thought Cabinet — Incubation & Crystallization

**System:** [60-chaos-engine](./SYSTEM.md)  
**Source:** `gzmo-chaos/src/thoughts.rs`

---

## Capability

The Thought Cabinet implements a **Disco Elysium-style internalization mechanic**: lore and skill outputs may be absorbed (≈18% chance when chaos roll ≥0.82), incubate for category-specific tick counts, impose cognitive drain, then **crystallize** into permanent mutations (gravity, friction, Lorenz ρ, tension bias). The engine becomes autopoietic — it learns from its own emissions.

---

## How it works

### Absorption

```78:103:gzmo-chaos/src/thoughts.rs
    pub fn try_absorb(&mut self, category: &str, text: &str, current_tick: u64, chaos_roll: f64) -> bool {
        if chaos_roll < ABSORB_THRESHOLD { return false; } // 0.82
        // MAX_SLOTS = 5; incubation_period(category) → 8–60 ticks
    }
```

Categories include joke, quote, fact, poem, story, card, dice_crit, sound, persona.

### Crystallization mutations

```134:150:gzmo-chaos/src/thoughts.rs
    fn crystallize(&mut self, thought: &IncubatingThought) -> CrystallizationEvent {
        let mutation = match thought.category.as_str() {
            "joke" => { self.mutations.gravity_mod -= 0.1; self.mutations.lorenz_rho_mod -= 0.2; /* ... */ }
            // quote +0.3 ρ, story +0.5 ρ, persona +0.8 ρ, etc.
```

`apply_rho_restoration` in pulse loop decays ρ_mod (linear k or tanh restore).

### Cognitive load

`active_drain_multiplier()` and `active_lorenz_noise()` increase while slots occupied — coupling cabinet to engine drain and σ noise.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Inputs | Auto-lore from pulse (every 30 ticks), skill `ChaosEvent` thought seeds |
| Outputs | `Mutations` in every `ChaosSnapshot`; `CrystallizationEvent` on tick |
| Triggers | `TriggerCondition::Crystallization` fires notify |
| Persistence | In-memory only — lost on process restart |

---

## THINKING nodes

> **THINKING — thoughts.rs:try_absorb**
> - *Reviewed:* 5 slots, threshold 0.82, category-specific incubation periods.
> - *Insight:* Rare absorption prevents cabinet saturation from lore spam.
> - *Risk / limitation:* Full cabinet silently rejects new thoughts — no backpressure signal.
> - *Enhancement:* Snapshot field `thoughts_rejected_last_tick` [CT101-safe].

> **THINKING — thoughts.rs:crystallize ρ impulses**
> - *Reviewed:* Asymmetric Δρ by category; documented in CHAOS_RHO_CONTROL_MODEL.
> - *Insight:* Persona/story permanently shift dynamics — narrative identity matters.
> - *Risk / limitation:* Unchecked accumulation could destabilize Lorenz (mitigated by rebirth halving).
> - *Enhancement:* Cap total |lorenz_rho_mod| lower on production CT101 [CT101-safe].

> **THINKING — thoughts.rs:Mutations persistence**
> - *Reviewed:* Default in-memory; not loaded from CHAOS_STATE.json on restart.
> - *Insight:* Restart = soft reset of personality — may be intentional for ops.
> - *Risk / limitation:* Daemon restart loses long-run crystallization history.
> - *Enhancement:* Serialize mutations to CHAOS_STATE.json [GZMO-next].

---

## Advancement

- **CT101:** Monitor `thoughts_crystallized` in HEARTBEAT.md during long daemon runs.
- **GZMO-next:** Optional persistence + replay for lab chaos experiments.

---

## Enhancement backlog

1. **[CT101-safe]** Cabinet full / reject counter in snapshot.
2. **[CT101-safe]** Tighter ρ_mod clamp for CT101 production profile.
3. **[CT101-safe]** Crystallization details in Synapse payload (category, target).
4. **[GZMO-next]** Persist `Mutations` across daemon restart.
5. **[GZMO-next]** Operator `/cabinet` skill to inspect/evict incubating thoughts.
