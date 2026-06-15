---
type: entity
title: memory/YYYY-MM-DD.md
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# memory/YYYY-MM-DD.md

Type: CONCEPT

## From [[openclaw-part2|openclaw-part2]] (2026-06-08)
- The agent's unassailable long-term knowledge base.
- Insights that survive the Deep Phase of the sleep cycle are permanently promoted to this file.
- GZMO is instructed to read this file before answering any prompt or initiating its first morning Heartbeat.
- Ephemeral daily logs for short-term memory.
- All daily interactions, background tasks, and "silent turns" are funneled into these files.
- Searchable via hybrid search (FTS5 BM25 + sqlite-vec cosine similarity) with a 30-day Temporal Decay algorithm.

## From [[the-cognitive-architecture-of-openclaw-agents-micro02|the-cognitive-architecture-of-openclaw-agents-micro02]] (2026-06-09)
- Acts as the agent's ephemeral, append-only ledger of working memory.
- Records user chat transcripts, tool execution outputs, and sensory observations.
- Subjected to aggressive temporal decay algorithms.

## From [[the-cognitive-architecture-of-openclaw-agents-micro04|the-cognitive-architecture-of-openclaw-agents-micro04]] (2026-06-09)
- Associated with the openclaw-memory crate.
- Represents high-throughput, append-only volatile memory for 'silent turns'.
- Asynchronous I/O handled via tokio::fs::OpenOptions.
