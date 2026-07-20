# Cascading compiler — Executable Wisdom (positioning)

**Status:** Closed as docs/positioning (archaeology #11) — not a new binary.  
**Date:** 2026-07-20  
**Boundary:** Lab / living emit paths already exist. Never CT101→`data-next` vault import.

## One sentence

**Executable Wisdom** is the marketing name for the cascade GZMO already runs:
ingest → verify → promote → honeypot → ripen/core → optional wiki emit — not a second RAG store.

## Cascade (named)

```text
Any input → prep → extract → verify → promote → vault
       → qualify → honeypot
       → [ripen → knowledge_core]
       → [wiki emit / Knowledge Gardener]
```

| Stage | What it is | Where |
|-------|------------|--------|
| Distill / ingest | Extract + verify facts | `session-distill`, ingest engines |
| Honeypot | Curated distillate for recall / dream / spark | honeypot-gate + lifecycle |
| Ripen → core | Dense exportable knowledge | `scripts/ripen-knowledge-core.py`, MACHINE M5 |
| Wiki emit | Source pages / gardener layer | `WikiEngine::emit_source_page`, `WIKI.md` |

Research labels (LLM-Compiler, Cascading Honeypot, Executable Wisdom) map onto this path —
see vault archaeology spark artefacts and [ct101-vault-archaeology-2026-07-20.md](../research/ct101-vault-archaeology-2026-07-20.md).

## What we do not build

- A separate “cascading-compiler” little-tool organ
- Importing CT101 living vault into workstation `data-next`
- Replacing overnight `KgPromoter` with a parallel compiler binary

## Operator feel

You know the cascade worked when honeypot rows rise, ripen/core artifacts exist when opted in,
and wiki pages cite real sources — not when a new CLI appears in the catalog.
