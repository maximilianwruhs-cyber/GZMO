# Pantheon feat re-land (Unpark Wave 2)

**Source branch:** `origin/feat/context-compress-headroom`  
**Doctrine:** [CHAOS_LIVING_VS_RITUAL.md](./CHAOS_LIVING_VS_RITUAL.md) — chaos off CT101 living KPI.  
**Do not invent** ghost `DICE_MASTER_*` symbols.

## Checklist

1. [x] Inventory Slice A/B/C vs thin main stubs
2. [x] **Slice A.0** — `data/dice_events.toml` + `dice_corpus` on main skill trait (no attractor/dispatch stack)
3. [x] **Slice A.1** — `dice_cascade` plan-only (TOML → suggested `/skill`; no nested execute)
4. [ ] Slice A full — nested execute + `dice_loop` / `card_forge*` / story — needs co-land of `attractor_common` + `dispatch` + `generative` from feat (**no** Slice C in same PR)
5. [x] Keep chaos off CT101 living KPI (plan-only cascade does not change daemon brain)
6. [x] `bash scripts/pantheon-ritual-check.sh` → feat-stack PASS (A.0)
7. [ ] Living faithfulness + takeaway-recall still PASS (after merge)
8. [ ] Skills bridge docs updated

Thin `/card` `/story` remain main stubs until full Slice A. `/dice` narratives + wild-magic **plan** come from TOML.  
Demos: `bash scripts/verify-dice-corpus.sh` · `bash scripts/verify-dice-cascade.sh` · `bash scripts/pantheon-ritual-demo.sh`.

**Blocker note:** feat nested cascade / `card.rs` import `attractor_common`, `dispatch`, `feedback_ipc`, pedagogy auto-triggers — not present on main. Do not blind-checkout those skill files.

## Slice A — ritual skills (preferred first PR)

| Path | Notes |
|------|--------|
| `gzmo-core/src/skills/dice.rs` | Full dice vs thin main stub |
| `gzmo-core/src/skills/dice_cascade.rs` | Cascade engine |
| `gzmo-core/src/skills/dice_corpus.rs` | Corpus helpers |
| `gzmo-core/src/dice_loop.rs` | Loop driver |
| `gzmo-core/src/skills/card.rs` + `card_corpus.rs` | Card skill |
| `gzmo-core/src/skills/card_forge.rs` + `card_forge_brief.rs` | Forge (large) |
| `gzmo-core/src/skills/story.rs` + `story_brief.rs` | Story v2 |
| `gzmo-core/src/skills/pkm_forge.rs` + `pkm_forge_brief.rs` | PKM forge |
| `data/dice_cascade.toml` | Cascade config |
| `data/dice_events.toml` | Events tier |
| `skills/skill_{dice,card,story}.sh` + `skills/{cardforge,pkmforge}.toml` | Shell bridges |
| `scripts/verify-dice-{cascade,corpus}.sh` | Lab verifies |
| `scripts/generate-dice-events-toml.py` | Generator |

## Slice B — docs / research

- `docs/{WUERFEL_DICE_LOOP,DICE_EVENTS_TIER_HANDOFF,CARD_FORGE_MASTER_HANDOFF,PKM_FORGE_MASTER_HANDOFF,STORY_SKILL_V2_SPEC,PANTHEON_FINAL_PACK,SKILL_PANTHEON_STANDARDIZATION_HANDOFF}.md`
- `research/story-*.md` + `research/story-baseline-metrics.json`

## Slice C — chaos engine (separate lab/TUI PR only)

- `gzmo-chaos/src/{chaos,engine,feedback,feedback_ipc,pedagogy_oscillator,pulse,thoughts,triggers}.rs`
- `gzmo-cli/src/chaos_{bootstrap,skill_cmd}.rs`
- `gzmo-core/src/chaos/lib.rs`
- `scripts/pi/chaos_skill.sh`, `skills/visuals/chaos_art.py`

## Out of scope

- Wiki entity noise from name filters (`*card*` / `*chaos*` / `*forge*`)
- Context-compress / pedagogy / Obolus / discovery commits on the same feat branch
