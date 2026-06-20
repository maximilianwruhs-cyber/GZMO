---
type: entity
title: RadixCache
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# RadixCache

Type: SYSTEM

## From [drive-research-enhancing-local-ai-hypervisor-architecture](/entities/drive-research-enhancing-local-ai-hypervisor-architecture.md) (2026-06-08)
- Queries the radix tree to find the longest matching prefix.
- Evicts nodes using a configurable policy like Least Recently Used (LRU).
- Tracks reference counts (lock_ref) to protect active generation sequences.
- Is part of SGLang.
- Manages memory allocations in page-aligned structures.
- Uses eviction policies like LRU.
