---
type: entity
title: "arXiv Network Patterns"
created: "2026-06-20"
updated: "2026-06-20"
status: draft
sources: 1
tags:
  - research
  - arxiv
  - compliance
---

# arXiv Network Patterns

Live arXiv integration under Tier 2 network exception (`compliance.network_exceptions` includes `arxiv`).

## Skill surface

`skills/skill_arxiv.sh` (slash command `/arxiv`):

| Subcommand | Endpoint | Cache |
|------------|----------|-------|
| `search --query` | export.arxiv.org API | none |
| `harvest --set` | OAI-PMH ListRecords | `data/arxiv-cache/metadata.jsonl` |
| `ingest-harvest` | curated batches → `gzmo ingest` | vault/honeypot + wiki sources |
| `fetch --id` | export.arxiv.org API | none |
| `graph --id` | Semantic Scholar Graph API | none |
| `status` | local cache stats | — |

## Compliance

- Outbound HTTP always permitted when `arxiv` is in `network_exceptions`
- Rate sleep default 0.25s between OAI-PMH calls (arXiv TOS alignment)
- Retrieved metadata: `./skills/skill_arxiv.sh ingest-harvest` (builds curated MD, batches, `gzmo ingest`)

## Librarian alignment

Extends [arxiv-search-collector](/wiki/entities/arxiv-search-collector.md) from stub to operational collector feeding the honeypot pipeline.
