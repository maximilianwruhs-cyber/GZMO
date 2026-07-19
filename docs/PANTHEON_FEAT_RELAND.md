# Pantheon feat re-land (Unpark Wave 2)

**Source branch:** `origin/feat/context-compress-headroom`  
**Doctrine:** [CHAOS_LIVING_VS_RITUAL.md](./CHAOS_LIVING_VS_RITUAL.md) — chaos off CT101 living KPI.  
**Do not invent** ghost `DICE_MASTER_*` symbols.

## Checklist

1. [x] Inventory Slice A/B/C vs thin main stubs
2. [ ] Land Slice A (`dice_loop` / cascade / `card_forge*` / story) — **no** Slice C in same PR
3. [ ] Keep chaos off CT101 living KPI
4. [ ] `bash scripts/pantheon-ritual-check.sh` → prefer feat-stack PASS
5. [ ] Living faithfulness + takeaway-recall still PASS
6. [ ] Skills bridge docs updated

Thin main stubs (`dice`/`card`/`story`) remain the installable ritual surface until Slice A lands. Demo: `bash scripts/pantheon-ritual-demo.sh`.

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
