# Pokemon Card Forge — Legendary Skill Handoff

**Status:** Shipped (2026-06-14) — `/pkm` elevated to legendary tier (◆) alongside `/card` and `/dice`  
**Repo:** `survey_GZMO`  
**Knowledge base:** `skills/pkmforge.toml`  
**Corpus:** `data/skills/pkm_forge_corpus.jsonl` (append-only forge ledger)

---

## 1. What makes `/pkm` legendary

`/pkm` is the pantheon's Pokemon-flavored **Generative Structured** gold standard. It mirrors the MTG `/card` implementation at a CCL-4 generative level:

| Capability | `/pkm` (legendary) |
|------------|---------------------|
| CCL-4 attractor envelope | ✅ |
| Phase-driven creative lens | **Concept / Archetype / Balance Design** pipeline |
| Chaos-indexed constraint sparks | **status effect + name seed** from TOML |
| Full knowledge-base coupling | **`pkmforge.toml` end-to-end** (elements, categories, rarities, name fragments) |
| Persistent corpus | **`pkm_forge_corpus.jsonl`** every accepted forge |
| Structured evidence JSON | **`--json` / Pi `display_plain`** with card + forge metadata |
| Rarity-tier engine feedback | **EX Resonance** (`Custom` event: tension +3, energy −4) |

Registry marker: **◆** in `/help` (same tier as `/dice` and `/card`).

---

## 2. Operator usage

```bash
/pkm                          # chaos-picked category
/pkm pokemon                  # fixed Pokemon card category
/pkm trainer                  # Trainer card (Item/Supporter/Stadium)
/pkm energy                   # Basic or Special Energy
gzmo chaos skill pkm --json   # structured evidence for probes
```

**Valid categories:** Pokemon, Trainer, Energy

---

## 3. Forge pipeline

```
pkmforge.toml
  → resolve_pkm_category (TOML catalog)
  → build_selection
       ├─ element (chaos_index × 8)
       ├─ rarity (Common -> Secret Rare weights)
       ├─ forge_mode (Concept | Archetype | Balance from phase)
       ├─ sparks (status effect, name_seed)
       └─ set_code (PKM + serial mod 1000)
  → LLM (system + attractor user prompt)
  → validate_forged_pokemon (strict structural gate)
  → parse_pkm → ASCII frame
  → PkmForged (+ EX Custom if Ultra/Secret Rare)
  → append pkm_forge_corpus.jsonl
```

---

## 4. Display anatomy

```
┌─────────────────────────────────────────────────┐
  ⚡ POCKET FORGE · SET PKM042
  ⚡ Rare Pokemon · Concept Design · inv #42 · #42
  tick … · phase … · valence … · ρ …
  sparks: Paralyzed · Basic · "Voltix"
├─────────────────────────────────────────────────┤
  [ANSI TCG frame — EX banner if applicable]
├─────────────────────────────────────────────────┤
  ✦ MYTHIC EX RESONANCE … (ultra/secret only)
  crystallize: ~35 ticks → friction −0.03
└─────────────────────────────────────────────────┘
```

---

## 5. Chaos feedback

| Event | Tension | Energy | Thought Cabinet |
|-------|---------|--------|-----------------|
| `PkmForged` | 0 | −2 | `"Name (Element)"` · category `pkm` · 35 ticks |
| Ultra/Secret `Custom` | +3 | −4 | `"ULTRA EX FORGE: …"` · category `pkm_ex` · 45 ticks |
| Crystallized `pkm`/`pkm_ex` | — | — | friction −0.03 |

Wild Magic: `/dice` D20 14–16 (spark), 17–18 (crystallize), 19–20 (legendary) can cascade into `/pkm`.

---

## 6. File map

| Path | Role |
|------|------|
| `gzmo-core/src/skills/pkm.rs` | Skill trait, retry, corpus, evidence, ex resonance |
| `gzmo-core/src/skills/pkm_forge.rs` | TOML loader, sparks, prompts, validator, frame renderer |
| `gzmo-core/src/skills/pkm_forge_brief.rs` | Concept / Archetype / Balance forge lens |
| `gzmo-core/src/skills/pkm_corpus.rs` | JSONL append |
| `skills/pkmforge.toml` | Elements + categories + rarities + name fragments |
| `data/skills/.pkm_*` | Dedup ledgers + call serial |
| `data/skills/pkm_forge_corpus.jsonl` | Forge archive |

---

## 7. Verification

```bash
cd ~/Projects/_foundation-audit/survey_GZMO
cargo test -p gzmo-core pkm pkm_forge
./scripts/verify-skill-standard.sh pkm
gzmo chaos skill pkm pokemon --json | jq '.evidence.set_code, .evidence.ultra_or_secret'
tail -3 data/skills/pkm_forge_corpus.jsonl
```

---

## 8. Image generation — explicitly out of scope

No artwork pipeline is implemented or coupled to the `/pkm` forge. Same as MTG `/card`, it uses ASCII-only terminal frames to maintain deterministic layout, reliability, and speed.
