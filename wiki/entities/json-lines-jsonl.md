---
type: entity
title: JSON Lines (JSONL)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# JSON Lines (JSONL)

Type: CONCEPT

## From [[drive-research-hermes-session-storage-migration-analysis|drive-research-hermes-session-storage-migration-analysis]] (2026-06-08)
- Previously used for raw conversation transcripts.
- Exists as a backup for raw conversation transcripts.
- Used as raw transcripts of conversations in the Hermes framework.
- Stored in ~/.hermes/sessions/.
- Each message and tool call per session is sequentially logged.
- Operates on a 'per-session' basis where each turn is a serialized line.
- Used for cold tier storage in compressed archives.
