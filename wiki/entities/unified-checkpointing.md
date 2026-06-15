---
type: entity
title: Unified Checkpointing
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Unified Checkpointing

Type: CONCEPT

## From [[drive-research-hermes-system-untersuchung-und-erweiterung|drive-research-hermes-system-untersuchung-und-erweiterung]] (2026-06-08)
- Extends checkpointing to conversational segments.
- Requires synchronization between the shadow Git repository and the SQLite database.
- Aims to store the Git commit hash as a foreign key in the session_segments table.
- Replaces the /fork logic.
- Couples Git commit hashes directly to SQLite segment IDs.
- Forms the foundation for 'BoL Manifest Checkpoints'.
