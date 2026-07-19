# Pantheon Skills — Ritual / Lab Front Door

**Status:** Operator front door (2026-07-19)  
**Living SoT (chaos policy):** [CHAOS_LIVING_VS_RITUAL.md](./CHAOS_LIVING_VS_RITUAL.md)  
**Quality bar:** [SKILL_GOLDEN_STANDARD.md](./SKILL_GOLDEN_STANDARD.md)  
**Bridge:** [SKILLS_BRIDGE.md](./SKILLS_BRIDGE.md)

## What this is

GZMO skills are bounded operator actions through the Rust registry (`dispatch_skill` → `Skill` trait), not chat macros. Full definition: [SKILL_GOLDEN_STANDARD.md](./SKILL_GOLDEN_STANDARD.md).

This front door covers the **legendary pantheon packs** recovered from `feat/context-compress-headroom` as **research archive** — not CT101 living success criteria.

## Where it runs

| Surface | Role |
|---------|------|
| **Ritual / lab** | Workstation chat/TUI + PulseLoop (chaos on) — where pantheon skills belong |
| **CT101 living** | Chaos-free mentor / discover-only scout — do **not** treat pantheon polish as living KPI |

See [CHAOS_LIVING_VS_RITUAL.md](./CHAOS_LIVING_VS_RITUAL.md).

## Legendary surfaces

| Surface | Kind | Notes |
|---------|------|-------|
| `/dice` | Mechanical | Tier math + event tables (archive) |
| `/card` | Generative structured | Card Forge contract (archive); main has thinner Color Pie path |
| `/story` | Generative CCL-4 | V2 acceptance criteria (archive-spec) |
| `/transform` | Mutation | Definitive Dozen personas (archive decision) |

## Main today vs feat stack

**On `main`:** thin / current stubs in `gzmo-core/src/skills/{dice,card,story}.rs`.

**On feat (not merged):** full attractor/forge/corpus stack (`attractor_common`, `card_forge*`, `dice_corpus`, `data/dice_events.toml`, …). Re-land only via a **separate ritual PR** — not this packaging pass.

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
| Thin `/dice` `/card` `/story` on main | Present |
| Feat attractor / Würfel / cascade stack | HOLD until dedicated ritual PR re-lands code |
| Ghost `DICE_MASTER_*` | Never existed — do not invent |

Legendary packs = research archive + ritual check; living KPI unchanged.
