> **Recovered 2026-07-19** from `origin/feat/context-compress-headroom`.
> **Research archive — not living CT101 doctrine.** See [PANTHEON_SKILLS.md](../../PANTHEON_SKILLS.md).
> Inventory: [LOST_KNOWLEDGE_INVENTORY.md](../../LOST_KNOWLEDGE_INVENTORY.md).

# Skill Pantheon Standardization — Implementation Handoff

**Status:** Shipped (2026-06-13) — framework + CCL-4 generative hexad (`story`, `poem`, `joke`, `card`, `word`, `define`)  
**Decision:** `/story` V2 (Attractor Fiction) is the generative template; `/dice` remains mechanical gold; `/card` keeps Card Forge fidelity  
**Repos:**

| Repo | Role |
|------|------|
| `/opt/gzmo/current (ritual/lab clone; see CT101_PATH_AUTHORITY.md)` | Rust skills, chaos engine, `cardforge.toml`, verify script |
| `~/gzmo_skills` | `BRIDGE.md` bridge + discovery cron (uses `gzmo_chaos`) |
| `~/.pi/agent/skills/gzmo-integration` | Pi `gzmo_chaos` tool + `SKILL.md` |

**Canonical spec:** [`SKILL_GOLDEN_STANDARD.md`](../../SKILL_GOLDEN_STANDARD.md)  
**Related:** [`GZMO_CHAOS_AGENT_GUIDE.md`](../../GZMO_CHAOS_AGENT_GUIDE.md), [`CHAOS_RHO_CONTROL_MODEL.md`](../../CHAOS_RHO_CONTROL_MODEL.md), [`../gzmo_skills/BRIDGE.md`](../../../gzmo_skills/BRIDGE.md)

---

## 1. Purpose

Operators and Pi agents were hitting `/story` twice and getting **identical prose** while the UI said `seed: "chaos"` — implying RNG. Root cause: generative skills used a **static LLM prompt** and ignored `ChaosSnapshot`, while chaos only adjusted temperature invisibly.

This handoff ships:

1. A **documented golden standard** (CCL levels, contracts, anti-patterns, PR checklist).
2. **Attractor Fiction** pattern for generative skills (live snapshot, `inv #N`, nonce, anti-repeat ledger, cabinet echo, crystallize footer).
3. **CCL badges** on `/help` (registry-driven).
4. **`verify-skill-standard.sh`** for repeatable QA.
5. Upgrades for **`/story`**, **`/poem`**, **`/joke`**, **`/card`**, **`/word`**, **`/define`** to **CCL-4** without dumbing down Card Forge.

---

## 2. Problem → fix (operator narrative)

| Symptom | Cause | Fix |
|---------|-------|-----|
| Same story on repeat | Identical user prompt + stable temperature | Nonce, `inv #N`, hash ledger, snapshot reload per attempt |
| “seed” confusing | Label was keyword, not RNG | Renamed to `keyword` / `motif` / `topic`; attractor coords in header |
| Chaos felt decorative | `ctx.chaos` unused in `story.rs` | `load_live_chaos_snapshot()` + phase/valence voice selection |
| Card lost shell richness | Rust path was thin vs `skill_card.sh` | `card_forge.rs` restores `cardforge.toml` + ASCII frame |

**Mental model for operators:** *Generative* means LLM-written, not *random every call*. Uniqueness requires **different prompts** or **explicit variation** — now built into CCL-4.

---

## 3. Architecture

### 3.1 Dispatch (unchanged contract)

```
/slash or gzmo_chaos({ command })
  → gzmo chaos skill <cmd> [args]
       → dispatch_skill() → Skill trait
            ├─ ChaosSnapshot (reload for generative)
            ├─ ChaosEvent → PulseLoop / feedback inbox
            └─ SkillOutput.display
```

### 3.2 CCL-4 generative stack

```
AttractorPromptInput
  ├─ live ChaosSnapshot (CHAOS_STATE.json, per attempt)
  ├─ call_serial (.<skill>_call_serial)
  ├─ nonce (tick ⊕ seed ⊕ serial ⊕ attempt ⊕ instant)
  ├─ incubating_previews (Thought Cabinet echo)
  └─ anti_repeat_hint (rolling SHA-256 ledger)

LLM → quality gate → display (ATTRACTOR header + body + crystallize footer)
```

### 3.3 Card Forge (preserved + coupled)

```
cardforge.toml → Color Pie, flavor tone, design method
chaos_index(snap) → color, rarity, default type
LLM structured fields → parse_card() → render_card_frame() (ASCII MTG frame)
+ ATTRACTOR FORGE header (inv #, tick, phase, valence, ρ)
```

---

## 4. Shipped inventory

### 4.1 New / major Rust modules (`gzmo-core/src/skills/`)

| File | Role |
|------|------|
| `skill_ccl.rs` | CCL-0…4 registry + `/help` badges |
| `attractor_common.rs` | Shared nonce, serial, hash ledger, display formatter |
| `story_brief.rs` | Phase → Hemingway/Kafka story modes |
| `poem_brief.rs` | Phase → poem modes |
| `joke_brief.rs` | Phase → BVT comedy modes |
| `card_forge.rs` | `cardforge.toml` loader, ASCII frame, forge prompts |
| `word_brief.rs` | Phase → neologism modes (CalmOrganic … SurrealAbsurd) |
| `define_brief.rs` | Phase → etymology modes (Poetic … Surreal) |
| `story.rs` | Attractor Fiction (rewritten) |
| `poem.rs` | Attractor Poetry (rewritten) |
| `joke.rs` | Attractor Comedy (rewritten) |
| `card.rs` | Attractor Forge (rewritten) |
| `word.rs` | Attractor Word (rewritten) |
| `define.rs` | Attractor Define (rewritten; dictionary API fallback) |
| `dispatch.rs` | `load_live_chaos_snapshot()`, `data_dir_from_skills()` |
| `help.rs` | CCL badges + gold star ★ |
| `registry.rs` | `help_entries_for_registry()` |

### 4.2 Chaos engine (`gzmo-chaos`)

| Change | File |
|--------|------|
| `incubating_previews: Vec<String>` on `ChaosSnapshot` | `pulse.rs` |

### 4.3 CLI

| Change | File |
|--------|------|
| Re-read snapshot before dispatch | `gzmo-cli/src/chaos_skill_cmd.rs` |
| Help entries with CCL | `chat.rs`, `tui/runner.rs` |

### 4.4 Shell delegates (Rust authoritative)

| Script | Status |
|--------|--------|
| `skills/skill_story.sh` | ✅ deprecated → `gzmo chaos skill story` |
| `skills/skill_poem.sh` | ✅ deprecated → `gzmo chaos skill poem` |
| `skills/skill_joke.sh` | ✅ deprecated → `gzmo chaos skill joke` |
| `skills/skill_card.sh` | ✅ deprecated → `gzmo chaos skill card` |
| `skills/skill_word.sh` | ✅ deprecated → `gzmo chaos skill word` |
| `skills/skill_define.sh` | ✅ deprecated → `gzmo chaos skill define` |
| All other `skill_*.sh` | ⚠️ legacy fallback via `shell_bridge` |

### 4.5 Runtime artifacts (`data/skills/`)

| File | Skill |
|------|-------|
| `.story_call_serial` / `.story_recent_hashes` | story |
| `.poem_call_serial` / `.poem_recent_hashes` | poem |
| `.joke_call_serial` / `.joke_recent_hashes` | joke |
| `.card_call_serial` / `.card_recent_hashes` | card |
| `.word_call_serial` / `.word_recent_hashes` | word |
| `.define_call_serial` / `.define_recent_hashes` | define |

### 4.6 Docs & tooling

| Path | Role |
|------|------|
| `docs/SKILL_GOLDEN_STANDARD.md` | Canonical quality bar |
| `docs/SKILL_PANTHEON_STANDARDIZATION_HANDOFF.md` | This file |
| `scripts/verify-skill-standard.sh` | Tiered verification |
| `skills/skills.toml` | Updated descriptions (story/poem/joke/card/word/define) |
| `~/.pi/agent/skills/gzmo-integration/SKILL.md` | Pi attractor + CCL notes |
| `~/gzmo_skills/BRIDGE.md` | Link to standard + verify script |
| `docs/GZMO_CHAOS_AGENT_GUIDE.md` | Tier 3 pass criteria updated |

---

## 5. Pantheon status (2026-06-13)

| CCL | Skills | Meets target? |
|-----|--------|---------------|
| **★ CCL-4** | `story`, `poem`, `joke`, `card`, `word`, `define` | ✅ Generative gold |
| **CCL-3** | `dice`, `transform`, `stabilize`, `ops`, `learn` | ✅ At/above mechanical/mutation bar |
| **CCL-2** | `poker`, `quote`, `sound`, `language` | ⚠️ Indexed only |
| **CCL-1** | `visual`, `calculate`, `help` | ⚠️ Passive / info |

**Not fully standardized yet:** 11 shell scripts still full legacy fallback; not all skills have dedicated unit tests.

---

## 6. Verification

### Quick (no Prime)

```bash
cd /opt/gzmo/current (ritual/lab clone; see CT101_PATH_AUTHORITY.md)
unset CARGO_TARGET_DIR
cargo test -p gzmo-core
cargo test -p gzmo-chaos
cargo build --release -p gzmo-cli
./target/release/gzmo chaos skill help | grep CCL-4
```

### Full golden-standard gate

```bash
./scripts/verify-skill-standard.sh          # all tiers
./scripts/verify-skill-standard.sh card    # one CCL-4 skill
```

**Tier 4** (live generative uniqueness) requires Prime:

```bash
curl -sf http://localhost:8000/v1/models
```

Expect: two consecutive calls → different body hash; header contains `inv #` and `ATTRACTOR`.

### Pi smoke

```
gzmo_chaos({ command: "help" })
gzmo_chaos({ command: "story" })
gzmo_chaos({ command: "card", args: "creature" })
```

---

## 7. Operator reference — CCL-4 display fields

| Skill | Header title | Seed label | Default seed | Crystallize |
|-------|--------------|------------|--------------|-------------|
| story | 📖 ATTRACTOR FICTION | keyword | `chaos` | ~40 ticks → +0.5 ρ_mod |
| poem | 🖋️ ATTRACTOR POETRY | motif | `verse` | ~25 ticks → +0.1 ρ_mod |
| joke | 😂 ATTRACTOR COMEDY | topic | `wit` | ~15 ticks → −0.2 ρ_mod |
| card | 🂡 ATTRACTOR FORGE | (color/rarity/type) | chaos-picked | ~35 ticks → friction −0.03 |
| word | 🔤 ATTRACTOR WORD | theme | (none — mode from phase) | ~45 ticks → friction −0.02 |
| define | 📚 ATTRACTOR DEFINE | term | required arg (e.g. `chaos`) | ~45 ticks → friction −0.02 |

---

## 8. Remaining work (next agent)

Priority order from [`SKILL_GOLDEN_STANDARD.md` §11](../../SKILL_GOLDEN_STANDARD.md):

| P | Task | Effort |
|---|------|--------|
| ~~P1~~ | ~~Raise **`word`** + **`define`** to CCL-4~~ | ✅ Shipped 2026-06-13 |
| P2 | Deprecate remaining **`skill_*.sh`** → delegate stubs (like `skill_card.sh`) | Low |
| P3 | Raise **`poker`**, **`quote`**, **`sound`** to CCL-3 | Medium |
| P4 | Extend **`verify-skill-standard.sh`** to all 17 skills + mechanical checks | Low |
| P5 | Optional: `ChaosEvent::StoryGenerated` metadata `{ keyword, tick, phase }` for Synapse | Medium |
| P6 | Optional: `/story continue` second beat | Medium |

**Do not regress:**

- Card Forge ASCII frame and `cardforge.toml` Color Pie
- Dice `chaos_roll` + tier events
- Thought Cabinet in-memory only (no vault writes from skills)

---

## 9. Session opener (paste to next agent)

```
You are continuing GZMO skill pantheon standardization.

Read first:
  docs/SKILL_PANTHEON_STANDARDIZATION_HANDOFF.md   (this handoff)
  docs/SKILL_GOLDEN_STANDARD.md                    (quality bar + matrix)
  docs/GZMO_CHAOS_AGENT_GUIDE.md                   (chaos QA tiers)

Repos:
  /opt/gzmo/current — Rust skills + verify script
  gzmo_skills  — BRIDGE.md
  ~/.pi/agent/skills/gzmo-integration/SKILL.md

Verify before claiming done:
  ./scripts/verify-skill-standard.sh

CCL-4 gold skills: story, poem, joke, card, word, define — do not strip attractor headers or Card Forge frame.
Next target: shell delegate sweep (P2), then raise poker/quote/sound to CCL-3 (P3).
```

---

## 10. Conversation lineage

This work originated from operator confusion: `/story` labeled `seed: "chaos"` but repeated identically. Investigation showed generative ≠ randomized per call. Evolution path:

1. **Story V2** — Attractor Fiction (`story_brief.rs`, hash ledger, `incubating_previews`)
2. **Golden standard** — CCL framework + compliance matrix
3. **CCL badges** — `/help` registry-driven
4. **`verify-skill-standard.sh`**
5. **Poem + joke** — CCL-4 via `attractor_common`
6. **Card** — CCL-4 while preserving Card Forge (finest legacy skill)
7. **Word + define** — CCL-4 via `word_brief.rs` / `define_brief.rs`; dictionary API fallback preserved for `/define`

Pi discovery cron pillar **B** probe `B07` (`gzmo_chaos` mechanical skill) and pillar **A** `A09` (`transform`) remain valid probe targets; all six CCL-4 generative skills now emit richer evidence (`inv #`, `ATTRACTOR`, tick).

---

*Handoff v1.1 — 2026-06-13. CCL matrix synced in `SKILL_GOLDEN_STANDARD.md`.*
