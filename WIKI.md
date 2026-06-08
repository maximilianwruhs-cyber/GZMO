---
title: WIKI — Schema and Conventions
version: "1.0"
---

# WIKI — Schema and Conventions

This document tells GZMO how the `wiki/` layer is structured and how to maintain
it. It is the wiki's configuration file: it makes the agent a disciplined wiki
maintainer, not a generic chatbot. You (operator) and GZMO co-evolve this doc
over time.

The `wiki/` directory is a **git-tracked, Obsidian-browsable markdown synthesis
layer** that sits between raw RAG retrieval (`gzmo_memory_search`, Qdrant) and
the chronological `DREAMS.md` consolidation. It is the persistent, compounding
artifact `data/vault.db` cannot be (SQLite is neither git-diffable nor
graph-viewable).

**Ownership:** GZMO writes and maintains every page. The operator curates
sources, asks questions, and reviews in Obsidian. The agent never invents facts
not backed by the vault or an ingested source.

**The Knowledge Gardener (Phase 2).** As of Phase 2 the wiki is tended by a
deterministic Rust `WikiEngine` rather than prompt jobs. Wear the **Knowledge
Gardener** hat for these duties (this is narrative framing — the machine job
keys stay `wiki_sync` / `wiki_lint`):

- **Emit** — on every ingest, `IngestEngine` hands the already-verified facts to
  `WikiEngine::emit_source_page`, which deterministically writes/updates the
  `sources/` + `entities/` pages and the catalog. No new LLM extraction.
- **Sync** (daily) — `WikiEngine::sync` re-grows `index.md` from the pages on
  disk so the catalog never drifts.
- **Lint** (weekly) — `WikiEngine::lint` walks the garden and reports orphans,
  broken `[[links]]`, missing frontmatter, and stale pages. Report-only; pruning
  stays human-directed.

These run as daemon loops on `[wiki].sync_cron_*` / `lint_cron_*`, or on demand
via `gzmo wiki <sync|lint|search|file-back|status>`.

**Retrieval is emit-only.** Wiki pages are *derived* from vault facts, so they
are **never re-ingested** into the honeypot (that would be circular). Read them
directly: `gzmo wiki search <q>` (CLI) or the read-only `gzmo_wiki_search` MCP
tool grep over `wiki/*.md`. Guards enforce this — anything under `wiki/` or
carrying `gzmo_synthetic: true` frontmatter is refused by the ingest pipeline and
the watcher.

## Layers

| Layer | Where | Mutability |
|-------|-------|------------|
| Raw sources | `inbox/`, knowledge folder, ingested docs | Immutable — read, never edited |
| Vault / KG | `data/vault.db`, Neo4j (`mcp__memory__*`) | Engine-owned (ingest/dream/spark) |
| **Wiki** | `wiki/` | **Agent-owned markdown — this doc governs it** |

## Directory layout

```
wiki/
  index.md            # content catalog (this is read first on every query)
  log.md              # append-only chronological op log
  entities/           # one page per person/system/org/product
  concepts/           # topics, ideas, comparisons, analyses
  sources/            # one summary page per ingested source
  assets/             # downloaded source images (gitignored)
```

## Page types and frontmatter

Every page starts with YAML frontmatter (consumed by Obsidian Dataview):

```yaml
---
type: entity        # entity | concept | source
title: "Canonical Page Title"
created: 2026-06-07
updated: 2026-06-07
sources: 0           # count of sources this page draws on
tags: []
status: draft        # draft | stable | stale
---
```

- **entity** (`wiki/entities/`) — a concrete thing: a person, system, org,
  product. Facts about it, with `[[wikilinks]]` to related entities/concepts and
  to the `sources/` pages that back each claim.
- **concept** (`wiki/concepts/`) — a topic, idea, comparison, or filed-back query
  answer. The evolving synthesis lives here.
- **source** (`wiki/sources/`) — a summary of one ingested raw source: key
  takeaways, and `[[wikilinks]]` to every entity/concept page it touched.

## Cross-reference convention

- Use Obsidian `[[wikilinks]]` for every internal reference.
- Every page links **its sources** and **at least one related page** — no
  orphans (a page with no inbound links is a lint finding).
- When a new source contradicts an existing claim, do not silently overwrite:
  flag it inline as `> CONTRADICTION (YYYY-MM-DD): <old claim> vs <new>` and
  reconcile, keeping the source links for both.

## Operations

### Ingest (emit)

`WikiEngine::emit_source_page` runs at the tail of the `extract -> verify ->
promote -> vault` pipeline (see `AGENTS.md`), gated by `[wiki].emit_on_ingest`.
It is deterministic and uses only already-verified facts:

1. Write/refresh the `wiki/sources/<slug>.md` summary for the ingested source.
2. Touch each entity page the facts reference, appending a provenance section
   (`## From [[source]]`) with that source's observations; add `[[wikilinks]]`.
3. Refresh `index.md` (add/repoint the catalog line + one-line summary).
4. Append `## [YYYY-MM-DD] ingest | <title>` to `log.md`.

Never write a fact not present in the vault or the source. The legacy
`wiki_sync` prompt job is retired (kept `disabled` in `gzmo.toml`).

### Query

1. Read `index.md` first to locate relevant pages.
2. Drill into those pages; synthesize an answer **with citations** to the
   `sources/` pages.
3. File notable answers back as a new `wiki/concepts/` page (`gzmo wiki file-back
   <title>`) so explorations compound (do not let them vanish into chat/Redis
   scratch). Append a `## [YYYY-MM-DD] query | <title>` entry to `log.md`.

Wiki retrieval is **emit-only**: `gzmo wiki search` / `gzmo_wiki_search` grep
over `wiki/*.md` with `index.md` as the cheap navigation layer. This is separate
from the honeypot RAG stack (Qdrant + `bge-reranker-v2-m3`), which indexes the
vault — wiki pages are never fed back into it.

### Lint

`WikiEngine::lint` runs weekly on `[wiki].lint_cron_*` (report-only; pruning
stays human-directed). It deterministically checks for:

- orphan pages (no inbound `[[links]]`),
- broken `[[links]]` (target page missing),
- missing or malformed frontmatter,
- stale pages (`status: stale`).

Findings go to `wiki/sources/_lint-YYYY-MM-DD.md`; a `## [YYYY-MM-DD] lint` entry
is appended to `log.md`. Deeper semantic checks (contradictions between pages,
claims superseded by newer sources, data gaps a web search could fill) remain a
gardener judgement call when reviewing the report. The legacy `wiki_lint` prompt
job is retired (kept `disabled` in `gzmo.toml`).

## Conventions summary

- The agent owns `wiki/`; the operator reviews in Obsidian.
- Frontmatter is mandatory on every page.
- Every claim is traceable to a `sources/` page.
- `index.md` and `log.md` are updated on every operation.
- Spark serendipity (`[spark]`) may drop a reflective `[[wikilink]]` note into a
  relevant page in addition to its `## Spark` section in `DREAMS.md`.
