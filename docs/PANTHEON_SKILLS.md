# Pantheon skills (front door)

**Status:** Operator front door (2026-07-19) · theater demo: [PANTHEON_DEMO.md](./PANTHEON_DEMO.md)  
**Living SoT (chaos policy):** [CHAOS_LIVING_VS_RITUAL.md](./CHAOS_LIVING_VS_RITUAL.md)  
**Quality bar:** [SKILL_GOLDEN_STANDARD.md](./SKILL_GOLDEN_STANDARD.md)  
**Bridge:** [SKILLS_BRIDGE.md](./SKILLS_BRIDGE.md)

Ritual / chat pantheon surface — **not** CT101 living KPI. Skills are bounded
registry actions (`dispatch_skill` → `Skill` trait), not chat macros.

**Demable:** `bash scripts/pantheon-ritual-demo.sh` → inventory + felt dice/card/story.

## Slash skills (Slice A on main)

| Skill | Role |
|-------|------|
| `/dice` | Corpus narratives + Wild Magic cascade (nested dispatch) |
| `/card` | Card forge (legendary pack path) |
| `/story` | Story skill (CCL-aware) |
| `/pkm` | PKM forge |
| `/poem` `/joke` `/word` `/define` | Generative attractor briefs |
| `/transform` | Definitive Dozen personas (archive decision) |

CCL badges render in `/help` via `gzmo-core/src/skills/skill_ccl.rs`.

## Ritual CLI (Slice C.0.1)

```bash
gzmo chaos skill help
gzmo chaos skill dice d20 --json
bash scripts/pi/chaos_skill.sh dice d20 --json
bash scripts/verify-chaos-skill.sh
```

One-shot lab runner: reads latest chaos snapshot, queues feedback for chat/TUI drain.
**Never** starts PulseLoop or the living daemon.

## Main today vs deferred

| Layer | Status |
|-------|--------|
| Slice A full (dispatch, cascade, forge, generative, CCL) | **On main** |
| Dice-loop **schedule** (`dice_loop.rs`) | On main; `dice.loop` default **off** |
| Slice C.0 feedback IPC + C.0.1 `chaos skill` | **On main** (ritual drain only) |
| Daemon `dice_loop` fire | Deferred (living risk) |
| Slice C.1 pedagogy oscillator | Deferred — Wave 2b after pantheon ritual |

Re-land inventory (historical + remaining C): [PANTHEON_FEAT_RELAND.md](./PANTHEON_FEAT_RELAND.md).

## Research archive

| Doc | Role |
|-----|------|
| [research/pantheon/SKILL_PANTHEON_STANDARDIZATION_HANDOFF.md](./research/pantheon/SKILL_PANTHEON_STANDARDIZATION_HANDOFF.md) | Inventory + verify map |
| [research/pantheon/CARD_FORGE_MASTER_HANDOFF.md](./research/pantheon/CARD_FORGE_MASTER_HANDOFF.md) | Legendary `/card` |
| [research/pantheon/DICE_EVENTS_TIER_HANDOFF.md](./research/pantheon/DICE_EVENTS_TIER_HANDOFF.md) | `/dice` tier math |
| [research/pantheon/STORY_SKILL_V2_SPEC.md](./research/pantheon/STORY_SKILL_V2_SPEC.md) | CCL-4 story acceptance |
| [research/pantheon/PANTHEON_FINAL_PACK.md](./research/pantheon/PANTHEON_FINAL_PACK.md) | Definitive Dozen personas |

## Unpark Wave 2 (ritual)

| Item | Status |
|------|--------|
| Front door + archive | Ready — `bash scripts/pantheon-ritual-check.sh` |
| Slice A full + C.0/C.0.1 | Landed on main |
| Daemon dice fire / C.1 pedagogy | HOLD — lab only; never overnight brain |
| Ghost `DICE_MASTER_*` | Never existed — do not invent |

Legendary packs = research archive + ritual check; living KPI unchanged.
