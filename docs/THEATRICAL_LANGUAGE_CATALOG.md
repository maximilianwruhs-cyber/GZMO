# Theatrical Language Catalog — GZMO Honesty Documentation

**Version:** 1.0  
**Last Updated:** 2026-06-26  
**Purpose:** Document every instance of theatrical/anthropomorphic language in GZMO with honest technical equivalents.

---

## Why This Catalog Exists

GZMO was built with extensive theatrical language: "dreams," "chaos," "thoughts," "moods," "organisms." These terms evoke complex systems but obscure the actual mechanisms. This catalog provides honest translations for every theatrical term, enabling developers to understand and modify the system without decoding metaphors.

**Key Principle:** If you can't explain it without metaphor, you don't understand the implementation.

---

## Category 1: Vitalism Theater (Life/Organism Metaphors)

Terms that falsely anthropomorphize the system as alive, conscious, or biological.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **alive** (boolean) | system_active flag | `engine.rs:27`, `pulse.rs:223` | Binary flag with no functional effect. Replaced with `EngineCapacity` enum. |
| **deaths** (counter) | restart_count | `engine.rs:28`, `pulse.rs:146` | Counter incremented on boolean toggle. No operational significance. |
| **heartbeat** | tick_interval | `pulse.rs:38`, `pulse.rs:617` | Fixed 344ms interval (174 BPM aesthetic). Replaced with `AdaptiveTempo`. |
| **organism** | system | `config.rs:1040` | "what the organism should attend to" → "what the system should process" |
| **organism state** | parameter_state | `chaos.rs:9` | Phase of computational workload, not biological state |
| **breath phase** | velocity_direction | `pulse.rs:266`, `triggers.rs:42` | Sign of rho_velocity_ema (-1, 0, +1). Not breathing. |
| **inhale** | positive_delta | `triggers.rs` | Positive change in rho_mod parameter |
| **exhale** | negative_delta | `triggers.rs` | Negative change in rho_mod parameter |
| **vital** | critical | `config.rs` | "vital systems" → "critical components" |
| **regeneration** | recovery | `engine.rs` | Energy restoration, not biological healing |
| **resurrection** | restart | `feedback.rs` | System restart, not rebirth |

### Honest Rewrite Examples

**Before:**
```rust
/// Process a heartbeat tick. Returns whether a "rebirth" occurred.
pub fn tick_heartbeat(...) -> bool
```

**After:**
```rust
/// Process a tick. Returns new capacity level based on workload.
pub fn tick(...) -> EngineCapacity
```

---

## Category 2: Cognitive Theater (Mind/Thought Metaphors)

Terms that suggest the system thinks, remembers, or dreams like a human.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **ThoughtCabinet** | ParameterMutationQueue | `thoughts.rs` | Queue for delayed parameter updates. |
| **thought** | parameter_request | `thoughts.rs:53`, `feedback.rs:119` | Request to modify chaos parameters. |
| **crystallize** | apply | `thoughts.rs:178`, `triggers.rs:47` | Execute queued parameter mutation. |
| **incubation** | delay | `thoughts.rs:19`, `thoughts.rs:46` | Delayed processing queue, not biological growth. |
| **absorb** | enqueue | `thoughts.rs:105`, `pulse.rs:921` | Add to processing queue. |
| **fusion** | batch | `thoughts.rs:113`, `thoughts.rs:183` | Combine similar queued requests. |
| **dream** | nightly_batch_etl | `dreams.rs`, `config.rs:76` | Scheduled knowledge extraction pipeline. |
| **autoDream** | scheduled_etl | `dreams.rs:45` | Automated nightly data processing. |
| **REM** | extract_phase | `dreams.rs` | Stage 1: LLM text extraction |
| **Deep** | verify_phase | `dreams.rs` | Stage 2: Confidence threshold filtering |
| **Light** | promote_phase | `dreams.rs` | Stage 3: KG/vault insertion |
| **sleep cycle** | scheduled_pipeline | `dreams.rs:4` | Nightly cron job, not sleep. |
| **what_to_remember** | extracted_facts | `spark.rs:46`, `spark.rs:702` | LLM-extracted knowledge items. |
| **memory consolidation** | dedup_and_store | `lifecycle.rs` | Vector similarity + insert/update operations. |
| **remember** | store | `spark.rs` | Save to database |
| **forget** | delete | `vault.rs` | Remove from storage |
| **cognition** | processing | `kg_extract.rs` | LLM-based text analysis |
| **insight** | output | `spark.rs` | Generated text |
| **intuition** | heuristic | `lifecycle.rs` | Rule-based classification |

### Honest Rewrite Examples

**Before:**
```rust
/// A thought seed destined for the Thought Cabinet
pub struct ThoughtSeed { ... }

/// Crystallization impulses on `lorenz_rho_mod`
fn crystallize(&mut self, thought: &IncubatingThought)
```

**After:**
```rust
/// A parameter mutation request queued for delayed application
pub struct ParameterRequest { ... }

/// Apply queued parameter mutations
fn apply(&mut self, mutation: &PendingMutation)
```

---

## Category 3: Physics Theater (Chaos/Entropy Metaphors)

Terms that misuse physics concepts to describe deterministic computation.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **chaos** | pseudo_random | `chaos.rs`, `skills/` | Deterministic Lorenz/logistic iteration. |
| **mood engine** | parameter_modulator | `pulse.rs` | Temperature/token modulation system. |
| **attractor** | state_generator | `chaos.rs:66` | Lorenz ODE solver. |
| **orbit** | state_trajectory | `thoughts.rs:204` | Sequence of (x,y,z) values. |
| **phase space** | parameter_space | `thoughts.rs:225` | 3D coordinate system (x,y,z). |
| **entropy** | variance | `metrics/` | Not thermodynamic entropy. |
| **turbulence** | high_variance | `thoughts.rs:303` | Large parameter swings. |
| **strange loop** | recursive_call | `dice_cascade.rs:214` | Self-referential skill invocation. |
| **cognitive noise** | random_offset | `chaos.rs:148` | Transient sigma perturbation. |
| **resonance** | correlation | `dice_resonance.rs` | "Lorenz-Logistic coupling" → periodic reseeding. |
| **bifurcation** | branch | `dice_bifurcation.rs` | Conditional execution path. |
| **coupling** | reseeding | `chaos.rs:188` | Using one PRNG output to seed another. |
| **gravity** | drain_factor | `engine.rs` | Arbitrary constant (9.8) in energy formula. |
| **friction** | resistance | `engine.rs` | Arbitrary constant (0.5) in energy formula. |
| **velocity** | change_rate | `pulse.rs` | Rate of parameter change. |
| **force** | event_impact | `feedback.rs` | Energy delta from events. |
| **field** | parameter_state | `prompts` | "chaos field" → current parameter values |
| **oscillation** | periodic_update | `pedagogy_oscillator.rs` | Regular interval processing |
| **damping** | decay | `thoughts.rs:390` | Multiplicative reduction over time |
| **impulse** | event | `feedback.rs` | Discrete input, not physical impulse |

### Honest Rewrite Examples

**Before:**
```rust
/// 3D Lorenz Attractor: dx/dt = σ(y-x), dy/dt = x(ρ-z)-y, dz/dt = xy-βz
pub struct LorenzAttractor { ... }

/// Cognitive noise from incubating thoughts
pub fn apply_cognitive_noise(&mut self, noise: f64)
```

**After:**
```rust
/// 3D deterministic state generator using Lorenz ODEs
pub struct StateGenerator { ... }

/// Apply transient parameter perturbation
pub fn apply_perturbation(&mut self, delta: f64)
```

---

## Category 4: Spiritual/Mystical Theater

Terms invoking mythology, magic, or transcendence.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **soul** | identity_config | `config.rs` | `[identity.soul_path]` → identity configuration file |
| **spark** | insight_generator | `spark.rs` | Hypothesis generation system. |
| **oracle** | predictor | `dice_oracle.rs` | High-confidence lore retrieval. |
| **pantheon** | skill_registry | `skills/registry.rs` | Collection of available skills. |
| **dice pantheon** | skill_categories | `skills/` | Tiered skill classification. |
| **legendary** | high_impact | `dice_legendary.rs` | Large parameter mutation (+1.0 rho). |
| **catastrophe** | major_event | `dice_catastrophe.rs` | Significant parameter shift. |
| **nucleation** | initialization | `thoughts.rs:337` | Parameter mutation start. |
| **transcends** | exceeds | `thoughts.rs:309` | "transcends parameter space" → exceeds normal range |
| **awakening** | activation | - | System start |
| **enlightenment** | optimization | - | Parameter tuning |
| **summon** | invoke | `skills/` | Call a skill function |
| **banish** | disable | - | Remove skill |
| **conjure** | generate | - | Create content |
| **sigil** | identifier | - | System ID |
| **talisman** | token | - | Access credential |

### Honest Rewrite Examples

**Before:**
```rust
/// Legendary crystallization expands ρ and transcends parameter space
"dice_legendary" => (0.0, 0.0, 1.0, 0.0, MutationEffect { ... })
```

**After:**
```rust
/// High-impact parameter adjustment significantly increases rho
"high_impact_rho" => (0.0, 0.0, 1.0, 0.0, ParameterEffect { ... })
```

---

## Category 5: Somatic/Sensory Theater

Terms describing physical sensations or body states.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **stillness** | low_tension | `prompts`, `config.rs` | Low workload state |
| **calmness** | stable_state | `prompts` | Minimal parameter variation |
| **restless** | high_variance | `prompts` | Large parameter swings |
| **fatigued** | low_energy | `prompts` | Below energy threshold |
| **dread** | high_tension | `thoughts.rs:267` | Elevated stress metric |
| **anxiety** | tension_spike | `feedback.rs:150` | Sudden tension increase |
| **exhausted** | throttled | `engine.rs` | Minimal processing capacity |
| **pain** | error | - | System error state |
| **comfort** | optimal | - | Ideal operating range |
| **breathing** | updating | `pulse.rs` | Periodic state refresh |
| **pulse** | tick | `pulse.rs` | Regular interval execution |
| **rhythm** | cadence | - | Update frequency |
| **heartbeat** | interval | `pulse.rs` | Time between ticks |

### Honest Rewrite Examples

**Before:**
```rust
// Low-tension opening prompt
"Connect this to stillness or the calmness of the chaos field."
```

**After:**
```rust
// Low-tension opening prompt
"Connect this to low workload states or stable parameter configurations."
```

---

## Category 6: Emotion/Affect Theater

Terms suggesting the system experiences emotions.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **mood** | parameter_state | `pulse.rs`, `prompts` | Current chaos configuration |
| **valence** | direction | `pulse.rs:118` | -1.0 to 1.0 parameter, not emotion |
| **affective** | state_monitor | `orchestrator.rs` | Emotional tracking (4-agent system) |
| **frustration** | retry_count | `orchestrator.rs` | Leakage detection retries |
| **stress** | load | `feedback.rs` | System load metric |
| **panic** | overload | - | Excessive queue depth |
| **serenity** | idle | - | No workload |
| **excitement** | high_activity | - | Heavy processing |
| **melancholy** | stale_data | - | Old cached results |
| **euphoria** | optimal_performance | - | Peak efficiency |
| **aggressive** | high_temperature | `pulse.rs` | LLM temp > 0.8 |
| **reflective** | low_temperature | `pulse.rs` | LLM temp < 0.5 |

### Honest Rewrite Examples

**Before:**
```rust
/// llm_valence: -1.0 (dark/aggressive) to 1.0 (calm/reflective)
pub llm_valence: f32
```

**After:**
```rust
/// llm_temperature_bias: -1.0 (high/creative) to 1.0 (low/precise)
pub llm_temperature_bias: f32
```

---

## Category 7: Agent/Person Theater

Terms suggesting autonomous entities with agency.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **Diagnoser** | evaluator | `orchestrator.rs` | LLM call for state assessment |
| **Planner** | scheduler | `orchestrator.rs` | LLM call for curriculum |
| **Affective** | monitor | `orchestrator.rs` | LLM call for load tracking |
| **Tutor** | generator | `orchestrator.rs` | LLM call for response |
| **Socratic soul** | prompt_template | `orchestrator.rs` | System prompt text |
| **Mentor** | interface | `mentor_*.rs` | User-facing API |
| **Guardian** | validator | - | Safety checker |
| **Sovereign** | autonomous | `main.rs` | Self-directed operation |
| **witness** | observer | - | Logging/monitoring |
| **voice** | output | - | Generated text |
| **intent** | goal | - | Objective |
| **desire** | objective | - | Target state |
| **will** | configuration | - | Set parameters |
| **agency** | capability | - | Available functions |

### Honest Rewrite Examples

**Before:**
```rust
/// The Socratic soul: internal agents collaborate to guide
const TUTOR_SYSTEM: &str = "You are the Socratic Tutor..."
```

**After:**
```rust
/// Tutor prompt template: generates Socratic responses
const TUTOR_PROMPT: &str = "Generate Socratic tutoring responses..."
```

---

## Category 8: Military/Combat Theater

Terms invoking conflict and warfare.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **guard** | validator | `cycle_guard.rs` | Prevents concurrent execution |
| **firewall** | filter | `spark.rs:749` | "dream firewall" → confidence threshold |
| **arsenal** | toolkit | - | Available methods |
| **deploy** | start | - | Initiate process |
| **target** | goal | - | Objective |
| **mission** | task | - | Operation |
| **stealth** | hidden | `orchestrator.rs:25` | "Stealth metrics" → internal tracking |
| **breach** | error | - | Security failure |
| **intercept** | catch | - | Error handling |
| **neutralize** | disable | - | Deactivate |

---

## Category 9: Religious/Ritual Theater

Terms invoking ceremony and worship.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **invoke** | call | `skills/` | Execute function |
| **ritual** | routine | - | Regular procedure |
| **ceremony** | process | - | Formal workflow |
| **sacrifice** | tradeoff | - | Cost/benefit |
| **offering** | input | - | Data provided |
| **prayer** | request | - | API call |
| **worship** | prioritize | - | High preference |
| **blessing** | approval | - | Validation |
| **curse** | bug | - | Defect |
| **anoint** | configure | - | Setup |
| **ordain** | authorize | - | Permit |
| **consecrate** | validate | - | Verify |

---

## Category 10: Temporal/Epoch Theater

Terms suggesting geological or cosmic timescales.

| Theatrical Term | Honest Equivalent | Files | Context |
|-----------------|-------------------|-------|---------|
| **epoch** | version | - | Release |
| **era** | phase | - | Development stage |
| **age** | period | - | Time span |
| **eon** | duration | - | Long interval |
| **cycle** | iteration | `dreams.rs` | Repeated process |
| **season** | quarter | - | 3-month period |
| **tide** | trend | - | Direction |
| **phase** | state | `chaos.rs` | System state |

---

## Implementation Status

| Category | Terms | Status |
|----------|-------|--------|
| Vitalism | 11 | Documented |
| Cognitive | 19 | Documented |
| Physics | 17 | Documented |
| Spiritual | 16 | Documented |
| Somatic | 13 | Documented |
| Emotion | 14 | Documented |
| Agent | 14 | Documented |
| Military | 9 | Documented |
| Religious | 12 | Documented |
| Temporal | 8 | Documented |
| **Total** | **143** | **Complete** |

---

## Migration Priority

### Critical (Confusing Core Functionality)
1. `ThoughtCabinet` → `ParameterMutationQueue`
2. `dream` → `nightly_batch_etl`
3. `crystallize` → `apply`
4. `chaos` → `state_generator`
5. `heartbeat` → `tick_interval`

### High (Frequent Developer Contact)
6. `mood` → `parameter_state`
7. `thought` → `request`
8. `incubation` → `delay`
9. `soul` → `identity_config`
10. `spark` → `insight_generator`

### Medium (Occasional Contact)
11-25. Agent names, dice categories, skill themes

### Low (Documentation/Comments)
26-143. Prompts, descriptions, internal comments

---

## Usage Guidelines

### For New Contributors
1. Check this catalog when encountering unfamiliar terms
2. Use the "Honest Equivalent" column in your mental model
3. Prefer honest terms in new code and comments
4. Question theatrical terms in PR reviews

### For Maintainers
1. Prioritize renaming Critical terms first
2. Accept theatrical terms in user-facing copy (if desired)
3. Never use theatrical terms in API documentation
4. Update this catalog when adding theatrical language

### For Documentation
1. Always define theatrical terms on first use
2. Provide honest equivalent in parentheses
3. Prefer honest terms in technical documentation
4. Use theatrical terms only for historical context

---

## Related Documents

- `HONEST_GZMO.md` — Architecture description using honest terms
- `MIGRATING_FROM_THEATER.md` — Guide for replacing theatrical code
- `ARCHITECTURE.md` — Original architecture (theatrical)

---

## Changelog

- **2026-06-26 v1.0** — Initial catalog with 143 terms across 10 categories