---
type: entity
title: "Open Knowledge Format"
created: "2026-06-20"
updated: "2026-06-20"
status: draft
sources: 1
tags:
  - research
  - okf
---

# Open Knowledge Format

Open Knowledge Format (OKF) v0.1 formalizes the LLM-Wiki pattern GZMO already uses: Markdown bundles with YAML frontmatter, `index.md` + `log.md`, and path-as-identity.

## GZMO alignment

- [Wiki Index](/index.md) declares `okf_version: "0.1"`.
- Page types: `entity`, `concept`, `source`, `index`, `log`, plus OKF extensions `runbook`, `topic`, `metric`.
- Wiki is **emit-only**; curated ingest uses `~/Schreibtisch/knowledge/curated/`.
- Internal links use CommonMark: bracket label plus path under `/entities/` or `/sources/` (global migration 2026-06-20).

## Type taxonomy decision (2026-06-20)

Handoff `hybrid-2026-06-20T09-28-14Z-socratic_brief.json` asked whether to extend OKF `type` enum or keep emit-only wiki.

**Decision:** **Hybrid emit-only + minimal enum extension.**

| Layer | Rule |
|-------|------|
| Wiki emit | Operators/agents write Markdown with YAML frontmatter; `type` uses the extended Rust `PageType` set. |
| OKF spec | v0.1 documents required fields; new types (`runbook`, `topic`, `metric`) are GZMO extensions, not a full OKF schema fork. |
| Ingest | Curated sources under `~/Schreibtisch/knowledge/curated/` remain the promotion path into vault/honeypot — wiki pages are not auto-ingested. |
| Migration | Wikilink→CommonMark completed 2026-06-20 (`scripts/migrate-wiki-wikilinks.py`); lint counts index CommonMark links. |

Rationale: extending `PageType` in `wiki_md.rs` gives compile-time exhaustiveness without forcing a global wikilink rewrite. Full OKF `type` enum parity is deferred until an external OKF v0.2 spec lands.

## Relationship

LINK: LLM-Wiki —formalizes→ OKF | Honeypot RRF —supplements→ OKF progressive disclosure
