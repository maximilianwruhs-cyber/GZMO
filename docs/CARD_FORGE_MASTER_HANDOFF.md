# Card Forge — Legendary Skill Handoff

**Status:** Shipped (2026-06-14) — `/card` elevated to legendary tier (◆) alongside `/dice`  
**Repo:** `survey_GZMO`  
**Knowledge base:** `skills/cardforge.toml` (NotebookLM TCG architecture notebook)  
**Corpus:** `data/skills/card_forge_corpus.jsonl` (append-only forge ledger)

---

## 1. What makes `/card` legendary

`/card` is the pantheon's **Generative Structured** gold standard. It exceeds CCL-4 generative skills in five ways:

| Capability | `/story` et al. | `/card` (legendary) |
|------------|-----------------|---------------------|
| CCL-4 attractor envelope | ✅ | ✅ |
| Phase-driven creative lens | Story modes | **Vision / Set / Play Design** pipeline |
| Chaos-indexed constraint sparks | keyword | **keyword + subtype + name seed** from TOML |
| Full knowledge-base coupling | brief files | **`cardforge.toml` end-to-end** (colors, types, rarities, keywords, fragments) |
| Persistent corpus | — | **`card_forge_corpus.jsonl`** every accepted forge |
| Structured evidence JSON | dice only | **`--json` / Pi `display_plain`** with card + forge metadata |
| Rarity-tier engine feedback | — | **Mythic Resonance** (`Custom` event: tension +3, energy −4) |

Registry marker: **◆** in `/help` (same tier as `/dice`).

---

## 2. Operator usage

```bash
/card                          # chaos-picked type
/card creature                 # fixed type
/card planeswalker             # now supported
gzmo chaos skill card --json     # structured evidence for probes
```

**Valid types:** creature, instant, sorcery, enchantment, artifact, planeswalker

---

## 3. Forge pipeline

```
cardforge.toml
  → resolve_card_type (TOML catalog)
  → build_selection
       ├─ color (chaos_index × 5)
       ├─ rarity (1:7:24:88 weights)
       ├─ forge_mode (Vision | Set | Play from phase)
       ├─ sparks (keyword, subtype, name_seed)
       └─ set_code (ATR + serial mod 1000)
  → LLM (system + attractor user prompt)
  → validate_forged_card (strict structural gate)
  → parse_card → ASCII frame
  → CardForged (+ Mythic Custom if mythic)
  → append card_forge_corpus.jsonl
```

---

## 4. Display anatomy

```
┌─────────────────────────────────────────────────┐
  🂡 ATTRACTOR FORGE · SET ATR042
  🌿 Rare Creature · Set Design · inv #42 · #42
  tick … · phase … · valence … · ρ …
  sparks: Trample · Golem · "Shadow Oracle"
├─────────────────────────────────────────────────┤
  [ANSI MTG frame — mythic banner if applicable]
├─────────────────────────────────────────────────┤
  ✦ MYTHIC RESONANCE … (mythic only)
  crystallize: ~35 ticks → friction −0.03
└─────────────────────────────────────────────────┘
```

---

## 5. Chaos feedback

| Event | Tension | Energy | Thought Cabinet |
|-------|---------|--------|-----------------|
| `CardForged` | 0 | −2 | `"Name (Type)"` · category `card` · 35 ticks |
| Mythic `Custom` | +3 | −4 | `"MYTHIC FORGE: …"` · category `card_mythic` · 45 ticks |
| Crystallized `card` | — | — | friction −0.03 |

Wild Magic: `/dice` D20 14–16 (spark), 17–18 (crystallize), 19–20 (legendary) can cascade into `/card`.

---

## 6. File map

| Path | Role |
|------|------|
| `gzmo-core/src/skills/card.rs` | Skill trait, retry, corpus, evidence, mythic resonance |
| `gzmo-core/src/skills/card_forge.rs` | TOML loader, sparks, prompts, validator, frame renderer |
| `gzmo-core/src/skills/card_forge_brief.rs` | Vision / Set / Play forge lens |
| `gzmo-core/src/skills/card_corpus.rs` | JSONL append |
| `skills/cardforge.toml` | Color Pie + types + rarities + keywords + name fragments |
| `data/skills/.card_*` | Dedup ledgers + call serial |
| `data/skills/card_forge_corpus.jsonl` | Forge archive |

---

## 7. Verification

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
cargo test -p gzmo-core card card_forge
./scripts/verify-skill-standard.sh card
gzmo chaos skill card creature --json | jq '.evidence.set_code, .evidence.mythic'
tail -3 data/skills/card_forge_corpus.jsonl
```

---

## 8. Image generation — explicitly out of scope

**Do not block `/card` on artwork.** The legendary UX is:

- Instant ASCII MTG frame (always works, terminal + Pi JSON)
- Structured card text + corpus + chaos feedback
- No PIL, chafa, Prime image API, or LLM art step in the forge path

Historical pain came from coupling forge to slow/flaky image pipelines (deps, paths, model calls, partial renders). That coupling is **intentionally removed**. `/visual` remains a separate optional skill (procedural Lorenz/sigil art via `chaos_art.py`) — it is not part of `/card`.

If art is ever revisited, requirements would be:

| Requirement | Rationale |
|-------------|-----------|
| Opt-in only (`/card --art` or post-forge hook) | Never slow the default forge |
| Procedural first (chaos-indexed PIL, no LLM) | Deterministic, offline, fast |
| Fail open | Missing deps → ASCII frame only, no error |
| Never gate quality on image success | Text forge is the product |

---

## 9. Optional polish (not mandatory for full UX)

These improve depth but are **not** required for operators to get value today:

| Item | Mandatory? | Why skip for now |
|------|------------|------------------|
| Color-pie lint (post-parse) | No | LLM + prompts already steer; lint is QA for perfectionists |
| `card_forge_highlights.toml` | No | Corpus JSONL already archives every forge |
| Set-block themes by phase | No | Forge lens + sparks already vary output |

Ship these only if playtesting shows repeated Color Pie violations or corpus noise.

---

## 10. Future (optional)

- Color-pie lint (post-parse keyword vs color weakness heuristics)
- `card_forge_corpus.toml` curated highlights (like `dice_events.toml`)
- Set-block themes from chaos phase (e.g. Idle → enchantment world)
