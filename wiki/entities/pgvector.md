---
type: entity
title: pgvector
created: 2026-06-08
updated: 2026-06-10
sources: 7
tags: []
status: draft
gzmo_synthetic: true
---







# pgvector

Type: SYSTEM

## From [obolus-vs-codium-extension-konzept-research-part2](/entities/obolus-vs-codium-extension-konzept-research-part2.md) (2026-06-08)
- Local RAG pipeline
- pgvector database
- Utilize LiteParse to inject structural metadata (e.g., file type, commit date, "Intelligence per Watt" scores) directly into pgvector columns.
- Combine BM25 (keyword) and pgvector (dense) results using a simplified Reciprocal Rank Fusion (RRF) instead of heavy cross-encoders.

## From [local-first-rag-architecting-sovereign-ai-with-li](/entities/local-first-rag-architecting-sovereign-ai-with-li.md) (2026-06-08)
- An extension for PostgreSQL used by Khoj AI for similarity search.

## From [obolus-micro05](/entities/obolus-micro05.md) (2026-06-09)
- Used with PostgreSQL for ServiceBot knowledge base.
- Supports embeddings and full-text search.

## From [the-cognitive-architecture-of-openclaw-agents-micro03](/entities/the-cognitive-architecture-of-openclaw-agents-micro03.md) (2026-06-09)
- An extension used by PostgreSQL.
- Used for cosine similarity matching.
- Enables simultaneous execution of vector similarity ordering and text match limits.

## From [the-cognitive-architecture-of-openclaw-agents-micro04](/entities/the-cognitive-architecture-of-openclaw-agents-micro04.md) (2026-06-09)
- Extension for PostgreSQL used for hybrid retrieval.
- Targets embedding vectors.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06.md) (2026-06-09)
- Extension for PostgreSQL.
- Provides native BM25 full-text search.
- Supports multi-instance deployments querying a shared knowledge base.

## From [the-agentic-operating-environment-a-synthesis-arc-micro01](/entities/the-agentic-operating-environment-a-synthesis-arc-micro01.md) (2026-06-10)
- Can be spun up via Docker/Podman.
