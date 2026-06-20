---
type: entity
title: /rollback
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# /rollback

Type: TOOL

## From [drive-research-hermes-system-untersuchung-und-erweiterung](/entities/drive-research-hermes-system-untersuchung-und-erweiterung.md) (2026-06-08)
- Controls the checkpoint infrastructure.
- Resets the file system to the state of the Nth checkpoint.
- Internally calls undo_last() after a file system rollback.
