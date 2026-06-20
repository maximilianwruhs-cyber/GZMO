---
type: entity
title: filesystem cache buffering
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# filesystem cache buffering

Type: CONCEPT

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- The Linux kernel relies on physical memory to handle essential OS processes, including filesystem cache buffering.
- Reducing vm.swappiness instructs the kernel to exhaustively reclaim inactive filesystem page caches before resorting to swapping out process memory.
