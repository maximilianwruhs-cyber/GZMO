# Pantheon feat re-land (Unpark Wave 2)

**Source branch:** `origin/feat/context-compress-headroom`  
**Doctrine:** [CHAOS_LIVING_VS_RITUAL.md](./CHAOS_LIVING_VS_RITUAL.md) — chaos off CT101 living KPI.  
**Do not invent** ghost `DICE_MASTER_*` symbols.

## Checklist

1. [x] Inventory Slice A/B/C vs thin main stubs
2. [x] **Slice A.0** — `data/dice_events.toml` + `dice_corpus` on main skill trait (no attractor/dispatch stack)
3. [x] **Slice A.1** — `dice_cascade` plan-only (TOML → suggested `/skill`; no nested execute)
4. [x] **Slice A full (PR #34)** — `dispatch` + nested `execute_cascade` + `attractor_common` / `generative` + `card_forge*` / story (**no** Slice C)
5. [x] **Dice-loop scheduling** — core schedules opt-in follow-up state; `dice.loop` remains default off
6. [x] `bash scripts/pantheon-ritual-check.sh` → feat-stack PASS
7. [x] Living faithfulness + takeaway-recall still PASS (post Slice A full merge)
8. [x] Skills bridge docs updated for A full
9. [x] **PKM Forge** — `/pkm` Rust skill, corpus, feedback, and cascade dispatch
10. [x] **Slice C.0** — lab `feedback_ipc` append/audit + snapshot-bridge ritual drain only
11. [x] **Slice C.0.1** — `gzmo chaos skill` runs a one-shot ritual/lab skill, reads the
    saved snapshot, and appends feedback to the C.0 inbox

**Landed:** attractor-style `/poem`, `/joke`, `/word`, and `/define` briefs.
**CCL registry:** `gzmo-core/src/skills/skill_ccl.rs` is on main; `/help` renders its badges.

**Still deferred:** daemon `dice_loop` fire wiring; Slice C.1 (pedagogy oscillator / pulse /
thoughts rework) and any living daemon chaos wiring. C.0.1 is a slim external CLI, not a
PulseLoop or daemon path; C.0 drains its lab inbox only.

**On main now:** Slice A full — `/dice` (corpus + nested cascade), `/card` forge, `/story`,
`/pkm`, generative briefs, CCL badges. Ritual demos:
`bash scripts/verify-dice-corpus.sh` · `bash scripts/verify-dice-cascade.sh` ·
`bash scripts/verify-chaos-skill.sh` · `bash scripts/pantheon-ritual-demo.sh`.

**Blocker note (Slice C.1 only):** do not blind-checkout feat `pedagogy_oscillator` /
auto-triggers into `daemon_cmd` or living overnight. Lab/TUI-only if landed.

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
| `gzmo-core/src/skills/pkm.rs` + `pkm_corpus.rs` + `pkm_forge.rs` + `pkm_forge_brief.rs` | Landed PKM forge |
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

### Slice C.0 — feedback IPC (landed, ritual drain only)

`feedback_ipc` serializes the main `ChaosEvent` set to
`state_dir/chaos_feedback_inbox.jsonl`; the chat/TUI snapshot bridge drains it into its existing
`feedback_tx`. It does not alter `daemon_cmd.rs`, start a daemon PulseLoop, or make CT101 living
chaos-active.

## Out of scope

- Wiki entity noise from name filters (`*card*` / `*chaos*` / `*forge*`)
- Context-compress / pedagogy / Obolus / discovery commits on the same feat branch
