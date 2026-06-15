---
type: entity
title: turnController.ts
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# turnController.ts

Type: TOOL

## From [[drive-research-hermes-session-storage-migration-analysis|drive-research-hermes-session-storage-migration-analysis]] (2026-06-08)
- Contains functions `flushStreamingSegment()` and `recordMessageComplete()`.
- These functions write reasoningText to the turn's persistent state without checking `showReasoning` status.
