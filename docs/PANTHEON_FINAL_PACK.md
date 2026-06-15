# Pantheon Final Pack — Locked Decision

**Date:** 2026-06-11  
**Authority:** Notebook `36ef9e7e` (Definitive Pantheon) + GZMO product fit  
**Status:** **Approved default** — adjust before Phase A content lands if you disagree

The notebook treats all 20 profiles as equal blueprints; it does not rank “essential” personas. This document applies **GZMO-specific** curation: mentor alignment, enforceable structural constraints, voice diversity, and content safety.

---

## Principle

`/transform` is a **generative-skill costume rack**, not core identity (`SOUL.md`). The final pack should be:

1. **Small enough** to maintain (quality gates, per-persona temp/constraints)
2. **Diverse enough** to stress-test the Rust persona pipeline
3. **Aligned** with Friendly Linux Mentor / technical operator use cases

---

## Final pack: **The Definitive Dozen** (12 personas)

### Tier 1 — Research core (8) — *ship in Phase A*

These are the notebook’s reason to exist: machine-readable constraints + calibrated sampling.

| # | Name | Category | Why keep |
|---|------|----------|----------|
| 1 | **Oliver Heaviside** | polymath | Telegraph/engineering voice; mandatory STEM vocabulary; fits Linux/sysadmin metaphor space |
| 2 | **Margaret Cavendish** | polymath | Ornate 17th-c prose; anti-reductionist; tests long-form generative skills |
| 3 | **Viktor Schauberger** | polymath | Biomimetic engineering; “do the opposite” heuristic; vortex/ecology jargon |
| 4 | **Alexander Grothendieck** | polymath | “Rising sea” abstraction; **refuses direct computation** — best leakage-style constraint test |
| 5 | **Sherlock Holmes (BBC)** | comedic | **Gold standard** for banned expressions + mandatory vocabulary + irony ratio |
| 6 | **Rick Sanchez** | comedic | High-volatility (0.88 / 0.95); stutter loops; STEM gaslighting trope |
| 7 | **Sterling Archer** | comedic | Low-temp precision (0.58 / 0.85); grammar-enforcement constraint |
| 8 | **Professor Farnsworth** | comedic | Bait-and-switch syntax; pairs with Rick as volatility bracket |

**Default sampling (polymaths — notebook silent):** `temperature = 0.70`, `top_p = 0.90`  
**Sherlock:** `0.65` / `0.85` (precision, not volatility table)

### Tier 2 — Hero anchors (4) — *keep from current `characters.toml`*

Retain the highest **constraint + recognition** heroes; drop the redundant six.

| # | Name | Why keep |
|---|------|----------|
| 9 | **Batman** | Terse tactical; zero warmth — contrasts mentor SOUL |
| 10 | **Spider-Man** | Quippy everyman; generative comedy baseline |
| 11 | **Wonder Woman** | Regal declarative; elevated diction |
| 12 | **Iron Man** | Technical wit + engineering refs; closest hero to mentor gear talk |

---

## Explicitly cut (8)

| Name | Reason |
|------|--------|
| Superman, Captain America, The Flash, Thor | Moral-boilerplate heroes; low constraint value vs Batman/Wonder Woman |
| Wolverine, Hulk | Voice gimmick (growl / HULK SMASH) without notebook-grade rules |
| **Eric Cartman** | Notebook spec embeds slurs/targets; incompatible with GZMO operator tool |
| **Avery Bullock** | Notebook spec requires explicit drug/sex content; same issue |

Cut heroes remain reachable via **LLM fallback** (`/transform CustomName`) once Phase A ports shell behavior.

---

## Category layout in `/transform` list

```
POLYMATH (4)
COMEDIC (4)
HERO (4)
```

---

## Phase A scope (when unlocked)

- Replace `characters.toml` content with 12 entries (not 20)
- Backfill `category`, `temperature`, `top_p`, `structural_constraints`, `banned_expressions`, `mandatory_vocabulary` for Tier 1 from notebook citations
- Tier 2 heroes: add `category = "hero"`; inherit engine default temp unless playtesting suggests overrides

---

## Not in pack (by design)

- Core mentor (`SOUL.md`) — never merged into Pantheon
- Custom/unknown characters — LLM fallback path
- TUI/daemon persona — unchanged; generative slash skills only
