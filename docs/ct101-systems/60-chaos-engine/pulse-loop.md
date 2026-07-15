# Pulse Loop — 174 BPM Heartbeat

**System:** [60-chaos-engine](./SYSTEM.md)  
**Source:** `gzmo-chaos/src/pulse.rs`

---

## Capability

`PulseLoop` is the unified chaos heartbeat at **174 BPM** (344ms/tick). Each tick drains skill feedback, advances Lorenz/logistic generators, ticks Thought Cabinet, updates engine energy/phase, computes LLM parameters, and broadcasts a **`ChaosSnapshot`** via `tokio::sync::watch`. Optional auto-lore from `lore.toml` feeds the cabinet and REPL notifications.

---

## How it works

### Snapshot schema

```36:71:gzmo-chaos/src/pulse.rs
pub struct ChaosSnapshot {
    pub tick: u64,
    pub x: f64, pub y: f64, pub z: f64,
    pub tension: f64, pub energy: f64, pub phase: Phase,
    pub llm_temperature: f32, pub llm_max_tokens: u32, pub llm_valence: f32,
    pub mutations: Mutations,
    pub rho_effective: f64, pub rho_mod_delta: f64,
    pub last_crystallization: Option<CrystallizationEvent>,
    // ...
}
```

### Main tick sequence

```346:517:gzmo-chaos/src/pulse.rs
    async fn run(mut self) {
        loop {
            interval.tick().await;
            // 1. drain feedback events
            // 2. lorenz.step(), logistic, cognitive noise, rho mutation
            // 3. engine.tick_heartbeat with effective gravity/friction
            // 4. cabinet.tick() → crystallizations
            // 5. rho restoration (linear or tanh)
            // 6. lorenz_to_temperature/tokens/valence
            // 7. snapshot_tx.send(snapshot)
            // hardware tension blend 90/10 from sysinfo thread
            // auto-lore every 30 ticks
        }
    }
```

### Start API

```284:344:gzmo-chaos/src/pulse.rs
    pub fn start(config: ChaosConfig) -> PulseHandle {
        // spawn sysinfo thread for hw_tension
        // tokio::spawn pulse.run()
        PulseHandle { snapshot_rx, feedback_tx, lore_rx, task, shutdown_flag }
    }
```

Comment in source: Synapse chaos heartbeat deferred until daemon-mode PulseLoop is permanent.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Config `[chaos]` | gravity, friction, seed, rho_decay_k, rho_restore_*, event chances, lore_path |
| Output files | `data/CHAOS_STATE.json`, `data/HEARTBEAT.md` (via bootstrap, every 15 ticks) |
| Channels | `watch::Receiver<ChaosSnapshot>`, `mpsc` feedback + lore |
| BPM | 174 → 344ms — matches legacy Randomizer |

---

## THINKING nodes

> **THINKING — pulse.rs:run loop**
> - *Reviewed:* Full tick ordering: feedback before physics before cabinet before snapshot.
> - *Insight:* Single writer for chaos state — no locks on snapshot consumers.
> - *Risk / limitation:* `watch` send drops if consumer lagging (only latest kept) — OK for LLM overrides.
> - *Enhancement:* Ring buffer of last N snapshots for Observatory [GZMO-next].

> **THINKING — pulse.rs:hw_tension thread**
> - *Reviewed:* sysinfo CPU+RAM blend at same 344ms cadence.
> - *Insight:* Couples host load to phase transitions (Idle/Build/Drop).
> - *Risk / limitation:* LXC cgroup CPU may not match host sysinfo on misconfigured CT101.
> - *Enhancement:* Use cgroup metrics when in container [CT101-safe].

> **THINKING — pulse.rs:LLM parameter mapping**
> - *Reviewed:* x→temperature [0.3,1.2], y→tokens [128,512], z→valence [-1,1].
> - *Insight:* Deterministic mapping from chaos coords — reproducible given seed.
> - *Risk / limitation:* Valence computed but not yet wired to gateway mood/system prompt.
> - *Enhancement:* Pass valence to gateway as soft tone hint [GZMO-next].

---

## Advancement

- **CT101:** Enable PulseLoop in `gzmo-daemon` full-time; write Synapse rho events from daemon bootstrap.
- **GZMO-next:** Optional chaos-disable mode for deterministic CI/regression runs.

---

## Enhancement backlog

1. **[CT101-safe]** Daemon startup: always `start_chaos_runtime` + `spawn_snapshot_bridge`.
2. **[CT101-safe]** Log tick overruns when interval slips >50ms.
3. **[CT101-safe]** Configurable BPM for test environments.
4. **[GZMO-next]** Snapshot history file for replay/debug.
5. **[GZMO-next]** Wire `llm_valence` into gateway system prompt modulation.
