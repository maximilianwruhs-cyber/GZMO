---
type: entity
title: SMB
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# SMB

Type: SYSTEM

## From [[drive-research-hermes-session-storage-migration-analysis|drive-research-hermes-session-storage-migration-analysis]] (2026-06-08)
- Network-attached file system that may lack native POSIX locking protocols.
- Can cause SQLite OperationalErrors if state.db is hosted on it.
