---
type: entity
title: sqlite-vec
created: 2026-06-08
updated: 2026-06-10
sources: 9
tags: []
status: draft
gzmo_synthetic: true
---









# sqlite-vec

Type: TOOL

## From [architectural-analysis-of-the-openclaw-ai-plugin-s](/entities/architectural-analysis-of-the-openclaw-ai-plugin-s.md) (2026-06-08)
- An embedded engine.
- Leveraged by plugins like basic-memory or lancedb.
- Used to construct a persistent memory vault.

## From [openclaw-part2](/entities/openclaw-part2.md) (2026-06-08)
- An extension used by OpenClaw within a localized SQLite database engine.
- Allows the agent to perform advanced hybrid search.
- A localized database engine used by OpenClaw for memory search.
- Relies on the performance characteristics of underlying block or file storage.

## From [openclaw-deep-research-part11-micro04](/entities/openclaw-deep-research-part11-micro04.md) (2026-06-09)
- An SQLite extension that can accelerate memory retrieval in OpenClaw.

## From [openclaw-part1-micro06](/entities/openclaw-part1-micro06.md) (2026-06-09)
- Used as the basis for a high-performance local database structure in OpenClaw.
- Has a 'sqlite-vec' C-extension for semantic vector search.
- C-extension for SQLite that enables semantic vector search.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02.md) (2026-06-09)
- Accelerates the memory_search tool.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06.md) (2026-06-09)
- SQLite extension used to accelerate vector similarity searches.
- Used for single-user deployments on Ubuntu 24.04.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro07](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro07.md) (2026-06-09)
- Used for querying local memory.
- Enables semantic search to inject established truths into context.
- Allows truth-finding to be independent of search engine algorithms.

## From [openclaw-part1-micro01](/entities/openclaw-part1-micro01.md) (2026-06-10)
- A C-extension used for vector search
- Enables searching by conceptual similarity via Cosine Similarity

## From [openclaw-part1-micro02](/entities/openclaw-part1-micro02.md) (2026-06-10)
- Used for semantic vector search
