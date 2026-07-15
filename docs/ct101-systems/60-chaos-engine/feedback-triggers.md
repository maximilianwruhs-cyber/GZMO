# Feedback & Triggers — Autopoietic Loop

**System:** [60-chaos-engine](./SYSTEM.md)  
**Sources:** `gzmo-chaos/src/feedback.rs`, `gzmo-chaos/src/triggers.rs`, `gzmo-cli/src/chaos_bootstrap.rs`

---

## Capability

Skills emit **`ChaosEvent`** feedback (dice, sound, poems, persona shifts, stabilize) into the pulse loop, modifying tension/energy and seeding the Thought Cabinet. **`TriggerEngine`** evaluates edge-triggered thresholds on each snapshot — firing notifications, skill runs, prompt injections, or meta-feedback. **`chaos_bootstrap`** wires pulse → gateway → files → synapse → triggers for chat/TUI/daemon entry points.

---

## How it works

### Feedback events

```28:91:gzmo-chaos/src/feedback.rs
pub enum ChaosEvent {
    DiceRoll { value: u8, max: u8 },
    SoundFired { category: SoundCategory },
    PoemGenerated { text: String },
    Stabilize { delta_rho: f64 },
    Custom { tension_delta: f64, energy_delta: f64, thought_seed: Option<ThoughtSeed> },
    // ...
}
```

Deltas computed in `tension_delta()`, `energy_delta()`, `thought_seed()` — e.g. nat-1 on d20 raises tension.

### Trigger engine

```148:181:gzmo-chaos/src/triggers.rs
    fn should_fire(&self, snap: &ChaosSnapshot, prev: &ChaosSnapshot) -> bool {
        // edge-triggered Above/Below; PhaseEnter; Crystallization; Death; Periodic
        // cooldown_ticks per trigger
    }
```

Defaults include tension_critical → RunSkill sound, energy_critical notify, phase_drop, death_event, crystallization, autonomous_pulse (520 ticks ≈3 min InjectPrompt), rho alerts.

### Bootstrap bridge

```42:186:gzmo-cli/src/chaos_bootstrap.rs
pub fn spawn_snapshot_bridge(/* snapshot_rx, gateway, feedback_tx, state_dir, ... */) -> JoinHandle<()> {
    // on snapshot change:
    //   gateway.set_chaos_overrides(temperature, max_tokens)
    //   every 15 ticks: CHAOS_STATE.json, HEARTBEAT.md, Synapse SenseChaosRho
    //   triggers.evaluate → Notify / RunSkill / InjectPrompt / EmitEvent
}
```

`start_chaos_runtime` loads `[chaos]` from `GzmoConfig` TOML.

---

## Interfaces

| Kind | Detail |
|------|--------|
| Config `[chaos]` | Full `ChaosConfig` deserialize in bootstrap |
| Skill API | Send to `PulseHandle.feedback_tx` or `ChaosRuntime.feedback_tx` |
| REPL hooks | `__TRIGGER_SKILL__`, `__TRIGGER_INJECT__` prefix messages |
| Synapse | `EventType::SenseChaosRho` — append-only, no consumer |
| Files | `data/CHAOS_STATE.json`, `data/HEARTBEAT.md` |

---

## THINKING nodes

> **THINKING — feedback.rs:ChaosEvent deltas**
> - *Reviewed:* Dice distance-from-midpoint tension; sound category signed deltas.
> - *Insight:* Skills become proprioceptive sensors for the chaos body.
> - *Risk / limitation:* Spammy skills could pump tension — no global rate limit on feedback.
> - *Enhancement:* Per-skill cooldown on feedback magnitude [CT101-safe].

> **THINKING — triggers.rs:with_defaults**
> - *Reviewed:* 7+ triggers with cooldowns; autonomous_pulse InjectPrompt every ~3 min.
> - *Insight:* Engine can compel agent reflection without user input — autonomy hook.
> - *Risk / limitation:* InjectPrompt in daemon needs agent loop subscriber — REPL-only today for some actions.
> - *Enhancement:* Wire InjectPrompt to orchestrator internal quest [GZMO-next].

> **THINKING — chaos_bootstrap.rs:spawn_snapshot_bridge**
> - *Reviewed:* Atomic write via .tmp rename; gateway lock per snapshot.
> - *Insight:* HEARTBEAT.md is Observatory-friendly human telemetry.
> - *Risk / limitation:* Comment says PulseLoop synapse deferred in daemon — partial telemetry gap.
> - *Enhancement:* Enable rho synapse append in daemon mode [CT101-safe].

---

## Advancement

- **CT101:** Complete daemon wiring for trigger RunSkill and InjectPrompt paths.
- **GZMO-next:** TOML-configurable trigger definitions instead of hardcoded defaults.

---

## Enhancement backlog

1. **[CT101-safe]** Daemon agent loop handles `__TRIGGER_INJECT__` and `__TRIGGER_SKILL__`.
2. **[CT101-safe]** Feedback rate limiter per skill name.
3. **[CT101-safe]** Always emit SenseChaosRho from daemon bootstrap.
4. **[GZMO-next]** `[chaos.triggers]` table in gzmo.toml.
5. **[GZMO-next]** Trigger actions → session_distill queue for autonomous reflections.
