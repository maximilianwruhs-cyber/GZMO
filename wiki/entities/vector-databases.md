---
type: entity
title: Vector databases
created: 2026-06-08
updated: 2026-06-09
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---






# Vector databases

Type: SYSTEM

## From [[the-cascading-honeypot-a-blueprint-for-executable|the-cascading-honeypot-a-blueprint-for-executable]] (2026-06-08)
- Underlying architecture for traditional RAG systems.
- Store information in a high-dimensional mathematical vector space.
- Information is frozen in the state it was found.

## From [[architectures-for-agentic-memory-virtual-context-micro04|architectures-for-agentic-memory-virtual-context-micro04]] (2026-06-09)
- Paired with Knowledge Graphs to extract, store, and retrieve long-term memory.
- Used by Zep, Mem0, and Cognee.

## From [[architectures-for-agentic-memory-virtual-context-micro07|architectures-for-agentic-memory-virtual-context-micro07]] (2026-06-09)
- Convert textual information into high-dimensional numerical arrays (embeddings).
- Retrieve data based purely on geometric proximity, known as cosine similarity.
- Flat vector architectures suffer from contextual isolation.

## From [[openclaw-deep-research-part10-micro05|openclaw-deep-research-part10-micro05]] (2026-06-09)
- Are thrown out by PageIndex.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro07|resilient-rust-based-mcp-client-and-llm-orchestrat-micro07]] (2026-06-09)
- Can be implemented to prevent context saturation.
