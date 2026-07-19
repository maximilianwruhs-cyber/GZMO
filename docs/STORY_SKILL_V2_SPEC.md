# `/story` V2 Specification — Chaos-Coupled Narrative Skill

This specification turns `/story` from a thin LLM wrapper into a **chaos-coupled narrative skill** where stories are visibly shaped by the Lorenz attractor, Thought Cabinet, and active Pantheon persona.

## 1. Success Criteria

1. **Uniqueness:** Consecutive calls with the same keyword within 30s must produce detectably different stories $\ge 95\%$ of the time.
2. **Coupling:** Display box shows at least 3 chaos fields (tick, phase, valence, $\rho$, incubating thoughts).
3. **Continuity:** Prompt references or continues a prior incubating thought in the cabinet if any exist.
4. **Persona Integration:** Active Pantheon characters override tone/voice without violating quality gates.
5. **Feedback Loop:** Crystallization of a story thought increases $\rho$ mod by $+0.5$, which is visible in the display footer / `/chaos` telemetry.
6. **Parity:** Rust skill is the single source of truth; shell bridge (`skill_story.sh`) delegates to `gzmo` CLI or mirrors logic.
7. **Safety:** All thought cabinet and story generation states remain in-memory (no raw writes to `vault.db`).

## 2. Architecture & Data Flow

```
/story keyword
   │
   ▼
Read ChaosSnapshot (with incubating_previews)
   │
   ▼
Build StoryBrief (compute nonce, derive StoryMode)
   │
   ▼
Generate System Prompt (Hemingway/Tension/Kafka based on StoryMode)
Generate User Prompt (inject keyword, tick, phase, valence, rho, nonce, cabinet echo, anti-repeat hint)
   │
   ▼
LLM Completion
   │
   ▼
Check rolling Anti-Repeat ledger
   ├── Match -> Retry with stronger anti-repeat hint (max 3 attempts)
   └── Unique -> Append hash to ledger, Emit StoryGenerated, Render display
```

## 3. Implementation Details

### 3.1 `ChaosSnapshot` Extension
Add the following field to `ChaosSnapshot` in `pulse.rs`:
```rust
pub incubating_previews: Vec<String>, // previews (first 80 chars) of active cabinet slots
```

### 3.2 `StoryBrief` Struct
Define in `gzmo-core/src/skills/story_brief.rs`:
```rust
pub enum StoryMode {
    HemingwayCalm,
    HemingwaySparse,
    RisingTension,
    KafkaSurreal,
}

pub struct StoryBrief {
    pub keyword: String,
    pub tick: u64,
    pub phase: Phase,
    pub valence: f32,
    pub temperature: f32,
    pub rho_effective: f64,
    pub nonce: u64,
    pub cabinet_echo: Option<String>,
    pub mode: StoryMode,
    pub anti_repeat_hint: String,
}
```

### 3.3 Dynamic Prompts
- **System Prompts:** Selects template from `StoryMode`. Focuses style strictly.
- **User Prompt:** Contains the keyword, tick, phase, valence, $\rho$, nonce, and cabinet echo. Instructs to avoid prior motifs (anti-repeat hint).

### 3.4 Anti-Repeat Ledger
Ledger file: `data/skills/.story_recent_hashes`. Keeps a rolling list of the last 20 generated story hashes. If a duplicate is detected during retry attempts, the prompt gets appended with specific warnings to avoid repeating motifs.

### 3.5 Display Format
```
┌─────────────────────────────────────────────────┐
  📖 ATTRACTOR FICTION
  keyword: chaos · tick 7710 · phase Drop · valence -0.41 · ρ 28.31
├─────────────────────────────────────────────────┤

  [story content here...]

├─────────────────────────────────────────────────┤
  incubating echo: "The clockwork mechanism snapped..."
  crystallize: ~40 ticks → +0.5 ρ_mod
└─────────────────────────────────────────────────┘
```
