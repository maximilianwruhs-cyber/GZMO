---
type: entity
title: vm.swappiness=10
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# vm.swappiness=10

Type: CONCEPT

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- A recommended setting for the vm.swappiness sysctl parameter for workstations running large memory-locked AI workloads.
- Instructs the kernel to exhaustively reclaim inactive filesystem page caches before resorting to swapping out process memory.
- A sysctl parameter that determines how aggressively the Linux kernel moves anonymous memory pages to swap storage.
- The default value on most desktop distributions is 60.
- For workstations running large memory-locked AI workloads, this parameter should be reduced to 10 or even 1.
