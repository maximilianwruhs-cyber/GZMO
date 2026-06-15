---
type: entity
title: shadow Git repository
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# shadow Git repository

Type: SYSTEM

## From [[drive-research-hermes-compression-and-bol-architecture|drive-research-hermes-compression-and-bol-architecture]] (2026-06-08)
- capable of atomic file and context rollbacks
- The Checkpoint Manager interfaces with a hidden Git repository.
- The shadow repository operates independently of the developer's actual .git folder.
- Leverages Git's content-addressable object database to deduplicate identical file states.
