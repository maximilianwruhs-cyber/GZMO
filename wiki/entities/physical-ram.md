---
type: entity
title: physical RAM
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# physical RAM

Type: CONCEPT

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- mlock() forces a virtual address range to remain resident in physical RAM.
- The Linux kernel manages physical memory dynamically.
- When the system's RLIMIT_MEMLOCK is exceeded, the system call fails.
- If an unprivileged process locks a multi-gigabyte virtual buffer, those pages are pinned directly into physical RAM and can never be evicted, paged out, or compressed by the kernel's memory management subsystem.
- If the physical RAM of the workstation is completely exhausted, the kernel's Out-of-Memory (OOM) subsystem is activated.
- Administrators must ensure that the combined size of locked model weights, KV cache, and execution thread overhead does not exceed 80% to 90% of the workstation's total physical RAM.
