---
type: entity
title: memory system
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# memory system

Type: SYSTEM

## From [[openclaw-deep-research-part10-micro05|openclaw-deep-research-part10-micro05]] (2026-06-09)
- Is a component that enables an autonomous agent to do meaningful work without constant supervision.

## From [[openclaw-deep-research-part11-micro04|openclaw-deep-research-part11-micro04]] (2026-06-09)
- Stores agent's preferences, ongoing projects, and communication style across days and weeks.
- Lives in plain Markdown files inside the agent workspace.
- Includes daily logs, MEMORY.md for long-term facts, SOUL.md for personality, and HEARTBEAT.md for proactive tasks.
- Uses a compaction process to summarize older conversation turns.
- Supports embedding-based search, optionally accelerated by sqlite-vec.
- Does not use external databases like Redis or Pinecone.
