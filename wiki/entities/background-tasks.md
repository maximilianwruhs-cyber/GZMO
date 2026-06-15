---
type: entity
title: Background Tasks
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Background Tasks

Type: CONCEPT

## From [[openclaw-deep-research-part11-micro06|openclaw-deep-research-part11-micro06]] (2026-06-09)
- The background task ledger tracks all detached work.
- Includes ACP runs, subagent spawns, isolated cron executions, and CLI operations.
- Tasks are records, not schedulers.
- Can be inspected using `openclaw tasks list` and `openclaw tasks audit`.
