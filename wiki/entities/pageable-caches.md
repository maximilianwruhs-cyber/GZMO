---
type: entity
title: pageable caches
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# pageable caches

Type: CONCEPT

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- If the kernel cannot reclaim enough memory from pageable caches, the OOM killer is activated.
- Reducing vm.swappiness instructs the kernel to exhaustively reclaim inactive filesystem page caches before resorting to swapping out process memory.
