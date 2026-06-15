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

## From [[architectural-analysis-of-the-openclaw-ai-plugin-s|architectural-analysis-of-the-openclaw-ai-plugin-s]] (2026-06-08)
- An embedded engine.
- Leveraged by plugins like basic-memory or lancedb.
- Used to construct a persistent memory vault.

## From [[openclaw-part2|openclaw-part2]] (2026-06-08)
- An extension used by OpenClaw within a localized SQLite database engine.
- Allows the agent to perform advanced hybrid search.
- A localized database engine used by OpenClaw for memory search.
- Relies on the performance characteristics of underlying block or file storage.

## From [[openclaw-deep-research-part11-micro04|openclaw-deep-research-part11-micro04]] (2026-06-09)
- An SQLite extension that can accelerate memory retrieval in OpenClaw.

## From [[openclaw-part1-micro06|openclaw-part1-micro06]] (2026-06-09)
- Used as the basis for a high-performance local database structure in OpenClaw.
- Has a 'sqlite-vec' C-extension for semantic vector search.
- C-extension for SQLite that enables semantic vector search.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02]] (2026-06-09)
- Accelerates the memory_search tool.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06]] (2026-06-09)
- SQLite extension used to accelerate vector similarity searches.
- Used for single-user deployments on Ubuntu 24.04.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro07|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro07]] (2026-06-09)
- Used for querying local memory.
- Enables semantic search to inject established truths into context.
- Allows truth-finding to be independent of search engine algorithms.

## From [[openclaw-part1-micro01|openclaw-part1-micro01]] (2026-06-10)
- A C-extension used for vector search
- Enables searching by conceptual similarity via Cosine Similarity

## From [[openclaw-part1-micro02|openclaw-part1-micro02]] (2026-06-10)
- Used for semantic vector search
