---
type: entity
title: Explicit I/O Syncing
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Explicit I/O Syncing

Type: CONCEPT

## From [prompt-agent-engineering-part2-micro01](/entities/prompt-agent-engineering-part2-micro01.md) (2026-06-09)
- Do not rely on OS or runtime lazy flushing.
- Mandate explicit sync calls after writes to ensure data hits disk before signaling success.
