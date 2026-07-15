# Lorenz Physics — Attractor & Engine State

**System:** [60-chaos-engine](./SYSTEM.md)  
**Sources:** `gzmo-chaos/src/chaos.rs`, `gzmo-chaos/src/engine.rs`

---

## Capability

The physics layer implements a **3D Lorenz attractor** (RK4 integration) coupled to a **logistic map**, with phase-dependent σ (convection) and mutable ρ from Thought Cabinet crystallizations. `EngineState` tracks energy, alive/dead, deaths/rebirths, and phase-derived drain multipliers — creating compulsion loops when energy hits zero.

---

## How it works

### Phase model

```26:54:gzmo-chaos/src/chaos.rs
impl Phase {
    pub fn from_tension(t: f64) -> Self {
        if t < 30.0 { Phase::Idle } else if t < 70.0 { Phase::Build } else { Phase::Drop }
    }
    pub fn drain_multiplier(&self) -> f64 {
        match self { Idle => 0.3, Build => 1.5, Drop => 3.0 }
    }
    pub fn lorenz_sigma(&self) -> f64 { /* 8 / 10 / 14 */ }
}
```

### Lorenz step (RK4)

```82:118:gzmo-chaos/src/chaos.rs
    pub fn step(&mut self) -> (f64, f64, f64) {
        // dx/dt = σ(y−x), dy/dt = x(ρ−z)−y, dz/dt = xy−βz
        // RK4 with dt=0.005
    }

    pub fn apply_rho_mutation(&mut self, rho_mod: f64) {
        self.rho = 28.0 + rho_mod;
    }
```

### Engine heartbeat

```32:82:gzmo-chaos/src/engine.rs
    pub fn tick_heartbeat(&mut self, tension: f64, gravity: f64, friction: f64, chaos_roll: f64, thought_drain_mod: f64) -> bool {
        self.phase = Phase::from_tension(tension);
        let drain = gravity * friction * 0.02 * self.phase.drain_multiplier() * thought_drain_mod;
        // regen inverse to energy level; Drop phase = no regen
        // death at energy 0; 30% rebirth if chaos_roll > 0.7
    }
```

Rebirth in pulse loop halves `lorenz_rho_mod` to cool runaway mutations.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Config | `[chaos] gravity`, `friction`, `initial_tension`, `seed` |
| Telemetry | `ChaosSnapshot.phase`, `energy`, `deaths`, `x/y/z` |
| Docs | `docs/CHAOS_RHO_CONTROL_MODEL.md` |
| Consumers | Pulse loop only — no external direct API |

---

## THINKING nodes

> **THINKING — chaos.rs:LorenzAttractor**
> - *Reviewed:* RK4 at dt=0.005; bounded orbits in 10k-step tests.
> - *Insight:* f64 precision chosen over f32 for stable long-run daemon sessions.
> - *Risk / limitation:* ρ mutations can push attractor into divergent regimes if unclamped.
> - *Enhancement:* Hard clamp ρ effective to [10, 45] [CT101-safe].

> **THINKING — chaos.rs:LogisticMap coupling**
> - *Reviewed:* Reseed from Lorenz every 10 ticks; drives chaos_val and lore selection.
> - *Insight:* Dual chaos sources reduce periodicity visible to skills.
> - *Risk / limitation:* Reseed frequency fixed — not config-driven.
> - *Enhancement:* Expose reseed interval in `[chaos]` [GZMO-next].

> **THINKING — engine.rs:tick_heartbeat**
> - *Reviewed:* Scaled drain for 174 BPM; thought_drain_mod from cabinet load.
> - *Insight:* DROP phase hemorrhage creates urgency triggers (energy_critical).
> - *Risk / limitation:* Rebirth RNG uses logistic output — not cryptographically random (fine here).
> - *Enhancement:* Log death/rebirth events to Synapse [CT101-safe].

---

## Advancement

- **CT101:** Tune gravity/friction on CT101 host profile (LXC memory limits affect tension).
- **GZMO-next:** Extract physics crate for unit tests without full pulse loop.

---

## Enhancement backlog

1. **[CT101-safe]** Synapse event on engine death/rebirth with tick + deaths count.
2. **[CT101-safe]** ρ effective clamp in `apply_rho_mutation`.
3. **[CT101-safe]** Export phase transition counts to HEARTBEAT.md.
4. **[GZMO-next]** Configurable RK4 dt for performance vs stability tradeoff.
5. **[GZMO-next]** Separate chaos profile presets (calm ops vs discovery wild).
