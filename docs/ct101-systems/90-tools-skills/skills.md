# Skills — Rust-Native Slash Command Engine

**Source:** `gzmo-core/src/skills/*.rs`  
**Parent:** [90-tools-skills/SYSTEM.md](./SYSTEM.md)

---

## Capability

Replaces legacy shell skill dispatch with in-process Rust handlers. Skills run on `/command` in the REPL, receive the live `ChaosSnapshot`, and can inject `ChaosEvent` feedback into the pulse loop — closing the autopoietic loop (display → chaos state → Thought Cabinet).

**CT101 note:** Headless daemon does not expose slash skills; they run on workstation `gzmo chat` / TUI. Chaos pulse on CT101 still consumes events if emitted from other sources.

---

## How it works

### Trait and registry

```61:114:github-clone/GZMO/gzmo-core/src/skills/mod.rs
#[async_trait]
pub trait Skill: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn skill_type(&self) -> SkillType;
    async fn execute(&self, ctx: SkillContext<'_>) -> Result<SkillOutput>;
}

pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn Skill>>,
}
```

`SkillOutput` carries terminal `display`, `feedback: Vec<ChaosEvent>`, and `inject_to_conversation` flag.

### Skill types

| Type | Skills | LLM needed |
|------|--------|------------|
| `Mechanical` | `dice`, `sound`, `poker`, `calculate`, `quote`, `visual` | No |
| `Info` | `help` | No |
| `Generative` | (shell legacy: `/poem`, `/joke`, …) | Yes — not yet ported |

### Dice — chaos-driven autopoietic loop

```82:91:github-clone/GZMO/gzmo-core/src/skills/dice.rs
fn chaos_roll(snap: &ChaosSnapshot, max: u8) -> u8 {
    let combined = (snap.chaos_val * 10000.0
        + snap.x.abs() * 100.0
        + snap.y.abs() * 10.0
        + snap.z.abs())
        .fract();
    let roll = (combined * max as f64).floor() as u8 + 1;
    roll.clamp(1, max)
}
```

D20 tier rolls emit real mechanical effects (`tension_delta`, `energy_delta`, `ThoughtSeed`) via `tier_mechanical_effect`.

### Help — registry introspection

```36:63:github-clone/GZMO/gzmo-core/src/skills/help.rs
        let builtins = [
            ("/quit",    "Exit GZMO (auto-saves session)"),
            ("/clear",   "Clear conversation history"),
            ("/chaos",   "Display chaos engine state dashboard"),
            // ...
        ];
        // ...
        lines.push(format!("{GOLD}├─ Shell Skills (legacy) ──────────────────────────────────────────┤{RESET}"));
        lines.push(format!("  {DIM}Remaining shell skills: /card /joke /poem /word /story /define /transform /language{RESET}"));
```

### Registered modules

| File | Command | Chaos feedback |
|------|---------|----------------|
| `dice.rs` | `/dice [d6\|d20]` | `DiceRoll` + tier `Custom` events |
| `sound.rs` | `/sound` | `SoundFired` by tension band |
| `poker.rs` | `/poker` | Hand deal from chaos shuffle |
| `quote.rs` | `/quote` | Lore pool from `lore.toml` |
| `calculate.rs` | `/calculate` | None (bc subprocess) |
| `visual.rs` | `/visual [mode]` | Spawns Python `chaos_art.py` + chafa |
| `help.rs` | `/help` | None |

---

## Interfaces

| Interface | Path / config |
|-----------|---------------|
| Registration | `gzmo-cli/src/chat.rs`, `gzmo-cli/src/tui/runner.rs` — `SkillRegistry::register` |
| Lore data | `../Randomizer/lore.toml`, `lore.toml`, `data/lore.toml` (quote skill) |
| Visual art | External `chaos_art.py` + `chafa` binary (workstation) |
| Sound | `sox` for audio synthesis (optional) |
| Feedback channel | `mpsc::Sender<ChaosEvent>` from chaos pulse loop |

---

## THINKING nodes

> **THINKING — skills/mod.rs:SkillType**
> - *Reviewed:* Four skill types drive display and feedback behavior.
> - *Insight:* Clean seam between mechanical chaos toys and future generative LLM skills.
> - *Risk / limitation:* Generative type unused in Rust registry — still shell-backed.
> - *Enhancement:* Port `/poem` and `/joke` as `SkillType::Generative` with gateway hook. [GZMO-next]

> **THINKING — skills/dice.rs:chaos_roll**
> - *Reviewed:* Deterministic-from-state roll using Lorenz coordinates + tick.
> - *Insight:* Same attractor position → same roll until pulse advances — reproducible chaos narrative.
> - *Risk / limitation:* Not cryptographically random; fine for game mechanic, not security.
> - *Enhancement:* Log dice outcomes to Synapse for Observatory dice panel. [CT101-safe]

> **THINKING — skills/dice.rs:tier_mechanical_effect**
> - *Reviewed:* D20 tiers 1,2,3,8,11,15,17,18,19,20 push real `ChaosEvent::Custom`.
> - *Insight:* Narrative text is not decorative — it mutates tension/energy/thought seeds.
> - *Risk / limitation:* Large event tables (100 D20 strings) increase binary size; no hot-reload.
> - *Enhancement:* Externalize event pools to TOML for pedagogy tuning. [GZMO-next]

> **THINKING — skills/visual.rs:external deps**
> - *Reviewed:* Shells out to Python + chafa for terminal graphics.
> - *Insight:* Keeps heavy rendering out of Rust hot path.
> - *Risk / limitation:* Fails silently on CT101 if Python/chafa absent (workstation-only today).
> - *Enhancement:* Pure-Rust ASCII fallback for headless environments. [GZMO-next]

> **THINKING — skills/help.rs:legacy shell list**
> - *Reviewed:* Documents unmigrated shell skills explicitly.
> - *Insight:* Honest operator UX — prevents "command not found" without explanation.
> - *Risk / limitation:* Shell skills may diverge from chaos feedback conventions.
> - *Enhancement:* Deprecation banner when shell skill invoked; route to Rust port. [CT101-safe]

---

## Advancement

| CT101 legacy | GZMO-next |
|--------------|-----------|
| Skills on workstation REPL only | Optional skill RPC from lab `cognition-smoke` recipes |
| 7 Rust skills + 8 shell legacy | Full Rust skill suite; shell scripts retired |
| Chaos feedback in-process | Same events could be emitted via Synapse from remote operators |

---

## Enhancement backlog

| Rank | Item | Tag |
|------|------|-----|
| 1 | Port high-traffic shell skills (`/poem`, `/card`) to Rust | [GZMO-next] |
| 2 | Synapse logging for skill-triggered `ChaosEvent`s | [CT101-safe] |
| 3 | Hot-reload event pools (dice lore) from `gzmo_skills/data/` | [GZMO-next] |
| 4 | Skill discovery API for Observatory séance panel | [GZMO-next] |
| 5 | Remove bc dependency in `calculate` — pure Rust eval | [GZMO-next] |
