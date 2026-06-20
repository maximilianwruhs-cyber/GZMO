---
type: entity
title: anonymous memory pages
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# anonymous memory pages

Type: CONCEPT

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- The vm.swappiness sysctl parameter determines how aggressively the Linux kernel moves anonymous memory pages to swap storage.
- Reducing vm.swappiness instructs the kernel to exhaustively reclaim inactive filesystem page caches before resorting to swapping out process memory.
