> **Recovered 2026-07-19** from `origin/feat/context-compress-headroom`. See [LOST_KNOWLEDGE_INVENTORY.md](./LOST_KNOWLEDGE_INVENTORY.md).

# GZMO Skill Golden Standard

**Status:** Canonical quality bar for the Rust skill pantheon (19 slash commands)  
**Repo:** ritual/lab GZMO clone (path era was `survey_GZMO`; living paths: [CT101_PATH_AUTHORITY.md](./ops/CT101_PATH_AUTHORITY.md))  
**Reference implementations:** `/story` (Generative V2), `/dice` (Mechanical), `/card` (Generative structured), `/transform` (Mutation)  
**Front door:** [PANTHEON_SKILLS.md](./PANTHEON_SKILLS.md)  
**Handoff (archive):** [`research/pantheon/SKILL_PANTHEON_STANDARDIZATION_HANDOFF.md`](./research/pantheon/SKILL_PANTHEON_STANDARDIZATION_HANDOFF.md) — shipped inventory, verification, remaining work

---

## 1. What a GZMO skill is

A GZMO skill is **not** a chat macro. It is a **bounded operator action** that:

1. Runs through the Rust skill registry (`dispatch_skill` → `Skill` trait)
2. Receives a live `ChaosSnapshot` via `SkillContext`
3. May emit one or more `ChaosEvent`s into the PulseLoop / Thought Cabinet
4. Returns structured terminal output the operator (or Pi agent) can cite as evidence

```
/slash → dispatch_skill()
           ├─ Rust Skill trait → ChaosEvent → PulseLoop → Thought Cabinet
           └─ shell_bridge (legacy fallback only)
```

**Golden rule:** Rust registry is authoritative. Shell scripts are deprecation stubs that delegate to `gzmo chaos skill <cmd>`.

---

## 2. The four skill types

Every skill declares exactly one `SkillType`:

| Type | Purpose | LLM required | Chaos coupling expectation |
|------|---------|--------------|----------------------------|
| **Mechanical** | Deterministic or chaos-indexed mechanics (dice, sound) | No | **Required** — output varies with attractor state |
| **Generative** | LLM-produced creative output (story, card, joke) | Yes | **Required** — prompt or metadata must include live chaos context |
| **Mutation** | Changes session/engine mode (transform, ops, learn) | Sometimes | **Required** — must emit mode-shift events |
| **Info** | Read-only display (help) | No | Optional |

---

## 3. Mandatory contract (all skills)

### 3.1 Rust surface

| Requirement | Detail |
|-------------|--------|
| `Skill` trait | `name()`, `description()`, `skill_type()`, `async execute()` |
| Registry | Registered in `build_chaos_skill_registry()` |
| Errors | Return `Err` only for true failures; user-facing arg errors → `Ok(SkillOutput { display: "✗ ..." })` |
| Args | Document in `description` + `skills.toml`; sensible defaults when args empty |
| No vault writes | Skills never write `vault.db`; crystallization is in-memory ρ only |

### 3.2 Chaos feedback

| Requirement | Detail |
|-------------|--------|
| Emit events | Every skill that affects the engine must push `ChaosEvent` via `ctx.feedback_tx` |
| Thought seeds | Generative outputs use typed events (`StoryGenerated`, `JokeGenerated`, …) so `thought_seed()` fires |
| Daemon IPC | `gzmo chaos skill` queues events to `data/chaos_feedback_inbox.jsonl` when daemon runs |
| Stabilize delta | Use `ctx.stabilize_delta_rho` for `/stabilize` — never hardcode |

### 3.3 Display

| Requirement | Detail |
|-------------|--------|
| Boxed output | Creative skills use `boxed_display()` or equivalent consistent frame |
| Evidence | Operator can see **what happened** without reading logs |
| No misleading labels | e.g. use `keyword:` not `seed:` unless it is an actual RNG seed |
| `inject_to_conversation` | `true` when the agent should remember the output; `false` for help/listings |

### 3.4 Snapshot freshness

| Requirement | Detail |
|-------------|--------|
| Read `ctx.chaos` | Every skill receives snapshot at dispatch |
| Generative reload | Generative skills **reload** `CHAOS_STATE.json` per attempt via `load_live_chaos_snapshot()` |
| Gateway sync | Before LLM call: `gateway.set_chaos_overrides(snap.llm_temperature, snap.llm_max_tokens)` |
| CLI entry | `chaos_skill_cmd.rs` re-reads snapshot immediately before dispatch |

### 3.5 Pi / external agents

| Requirement | Detail |
|-------------|--------|
| Invocation | `gzmo_chaos({ command, args })` → `gzmo chaos skill` |
| Documentation | `~/.pi/agent/skills/gzmo-integration/SKILL.md` lists command + display semantics |
| No TUI | Skills must not spawn `gzmo chat` REPL |

### 3.6 Tests

| Requirement | Detail |
|-------------|--------|
| Unit tests | At least: happy path or core helper, quality gate (generative), event inference (shell_bridge) |
| No live LLM in CI | Gate functions and prompt builders tested without Prime |
| `cargo test -p gzmo-core` | Must stay green |

---

## 4. Chaos coupling levels (CCL)

Use these levels to score existing and new skills:

| Level | Name | Criteria |
|-------|------|----------|
| **CCL-0** | Disconnected | Ignores `ctx.chaos`; identical output on repeat (legacy shell-only) |
| **CCL-1** | Passive | Reads snapshot for display only (shows tick in footer) |
| **CCL-2** | Indexed | Uses `chaos_index()` or `chaos_roll()` so output varies with coordinates |
| **CCL-3** | Coupled | Chaos fields shape prompts, structure, or mechanical outcomes |
| **CCL-4** | Autopoietic | CCL-3 + Thought Cabinet echo + anti-repeat + visible ρ/crystallize footer + typed `ChaosEvent` |

**Golden standard target:**

- Mechanical → **CCL-2 minimum**, **CCL-3** preferred  
- Generative → **CCL-4** ( `/story` V2 is the template)  
- Mutation → **CCL-3** (mode shift visible + event emitted)  
- Info → **CCL-1** acceptable  

---

## 5. Generative golden standard (Attractor Fiction template)

`/story` V2 defines the bar for all generative skills:

### 5.1 Prompt architecture

```
StoryBrief / <SkillBrief>
  ├─ keyword or args
  ├─ live ChaosSnapshot (reloaded per attempt)
  ├─ call_serial (monotonic invocation #)
  ├─ nonce (tick ⊕ keyword ⊕ serial ⊕ attempt ⊕ instant)
  ├─ optional cabinet_echo from incubating_previews
  └─ anti_repeat_hint from rolling hash ledger
```

### 5.2 Required behaviors

| # | Behavior |
|---|----------|
| G1 | Phase/valence (or equivalent) selects voice or structure |
| G2 | User prompt includes attractor coordinates — not keyword alone |
| G3 | Unique invocation identity (`inv #N` or equivalent) |
| G4 | Rolling dedup ledger for generative bodies (where repetition is confusing) |
| G5 | `try_generative` or equivalent: quality gate + ≤3 retries |
| G6 | Persona-aware via `build_system_prompt()` when transform active |
| G7 | Footer documents Thought Cabinet impact (incubation ticks, Δρ if known) |

### 5.3 Display header template

```
📖 ATTRACTOR FICTION          (or skill-specific icon + title)
keyword: <arg> · inv #N · tick T · phase P · valence V · ρ R
```

Adapt icon/title per skill (`🂡 CARD FORGE`, `😂 JOKE`, etc.) but keep **evidence density**.

---

## 6. Mechanical golden standard (Dice template)

`/dice` defines the bar for non-LLM skills:

| # | Behavior |
|---|----------|
| M1 | Outcome derived from `chaos_roll(snap, max)` — not `rand::random()` |
| M2 | Narrative variant from `pick_variant(snap)` — same roll ≠ same prose |
| M3 | Show live `tick` in display |
| M4 | Tier effects emit secondary `ChaosEvent`s (e.g. crit → `Custom` + thought seed) |
| M5 | Invalid args → friendly message, no panic |

---

## 7. Mutation golden standard (Transform / Ops / Learn)

| # | Behavior |
|---|----------|
| U1 | Persist state under `skills/` (persona, ops flag, learn session) |
| U2 | Emit typed event on enter **and** exit (`PersonaShift` / `PersonaCleared`) |
| U3 | Display names what changed and how to revert |
| U4 | Pedagogy skills use `GatewayRouter` when prep needs a different profile |

---

## 8. Pantheon compliance matrix (2026-06-13)

| Skill | Type | CCL | Rust | Shell delegate | Tests | Notes |
|-------|------|-----|------|----------------|-------|-------|
| **story** | Generative | **4** | ✅ | ✅ | ✅ | **Gold standard** — Attractor Fiction |
| **poem** | Generative | **4** | ✅ | ✅ | ✅ | Attractor Poetry |
| **joke** | Generative | **4** | ✅ | ✅ | ✅ | Attractor Comedy |
| **card** | Generative | **4** | ✅ | ✅ | ✅ | Attractor Forge — Color Pie + ASCII frame |
| **pkm** | Generative | **4** | ✅ | ✅ | ✅ | Attractor Forge — Element + ASCII frame |
| **dice** | Mechanical | **3** | ✅ | fallback | ✅ | Gold standard mechanical |
| **transform** | Mutation | **3** | ✅ | fallback | partial | Pantheon + custom persona |
| **word** | Generative | **4** | ✅ | ✅ | ✅ | Attractor Word — phase-driven neologism modes |
| **define** | Generative | **4** | ✅ | ✅ | ✅ | Attractor Define — phase etymology + dictionary fallback |
| **stabilize** | Mutation | **3** | ✅ | stub | partial | Uses config delta |
| **ops** | Mutation | **3** | ✅ | stub | partial | |
| **learn** | Mutation | **3** | ✅ | stub | partial | PedagogyInternal |
| **poker** | Mechanical | **2** | ✅ | fallback | partial | |
| **quote** | Mechanical | **2** | ✅ | fallback | partial | Lore pool |
| **sound** | Mechanical | **2** | ✅ | fallback | partial | |
| **visual** | Mechanical | **1** | ✅ | fallback | partial | |
| **calculate** | Mechanical | **2** | ✅ | fallback | ✅ | bc + chaos frame + JSON evidence |
| **language** | Mutation | **2** | ✅ | fallback | partial | |
| **help** | Info | **1** | ✅ | dynamic | partial | Registry-driven |

**Legend:** ✅ = meets bar for that column; partial = works but lacks dedicated tests or full doc.

---

## 9. Anti-patterns (never ship)

| Anti-pattern | Why |
|--------------|-----|
| Label keyword as `seed` | Implies RNG; confuses operators |
| Same LLM prompt on every call | Generative ≠ deterministic; breaks trust |
| Ignore `ctx.chaos` in generative skills | Wastes the autopoietic loop |
| Shell script as authoritative path | Split brain; stale temperature |
| Hardcoded LLM temperature | Bypasses Lorenz mood thermostat |
| Write vault / Neo4j from skills | Skills perturb ρ; memory tools ingest |
| Spawn chat TUI from Pi tools | Breaks headless agent workflows |
| Generic AI slop gates only | Need domain gates (story fairy-tale ban, poem abstract-word ban) |

---

## 10. Verification checklist (per skill PR)

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
./scripts/verify-skill-standard.sh          # full pantheon
./scripts/verify-skill-standard.sh story    # one generative skill
```

### Generative PR must pass

- [ ] Two consecutive calls → different body hash  
- [ ] Header shows `inv #N` or equivalent invocation identity  
- [ ] Header shows ≥3 chaos fields (tick, phase, valence, ρ)  
- [ ] `ChaosEvent` typed variant emitted  
- [ ] Quality gate rejects known slop patterns  
- [ ] `SKILL.md` updated if display format changed  

### Mechanical PR must pass

- [ ] Output references live tick  
- [ ] Repeated calls at different ticks can differ (where applicable)  
- [ ] `ChaosEvent` emitted  
- [ ] Invalid args handled gracefully  

---

## 11. Migration roadmap (pantheon → full gold)

**Shipped (2026-06-13):** poem, joke, card, pkm, word, define → CCL-4; seven generative skills now meet gold bar.

Priority order for remaining work:

1. **shell scripts** → all print deprecation + delegate (like `skill_story.sh`) — 11 legacy scripts remain  
2. **poker, quote, sound** → raise to CCL-3 (chaos shapes outcomes)  
3. **visual** → raise to CCL-2 (show tick, chaos-indexed variant if any)  
4. **skills.toml** → mark `handler` as `deprecated_delegate` in comments; registry is source of truth  
5. **Unified `/help`** → generate from registry metadata only (BRIDGE optional future work)  
6. Optional: `ChaosEvent` metadata for Synapse; `/story continue` second beat

---

## 12. Adding a new skill

1. Pick `SkillType` and target CCL (default: **CCL-3**, generative: **CCL-4**)  
2. Add `gzmo-core/src/skills/<name>.rs` implementing `Skill`  
3. Register in `registry.rs`  
4. Add `ChaosEvent` variant in `feedback.rs` + `thought_seed()` + `thoughts.rs` crystallize impulse  
5. Add unit tests (prompt builder, gates, display inference)  
6. Add `skills/skill_<name>.sh` delegate stub  
7. Update `skills.toml`, `SKILL.md`, this matrix  
8. Run verification checklist §10  

---

## 13. Related docs

| Doc | Role |
|-----|------|
| `docs/GZMO_CHAOS_AGENT_GUIDE.md` | Chaos engine operator + QA tiers |
| `docs/CHAOS_RHO_CONTROL_MODEL.md` | Crystallization Δρ table |
| `gzmo_skills/BRIDGE.md` | Pi discovery + platform bridge |
| `~/.pi/agent/skills/gzmo-integration/SKILL.md` | Agent tool surface |

---

*Golden standard v1.0 — 2026-06-13. Update the compliance matrix when a skill reaches the next CCL.*
