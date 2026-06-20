---
type: entity
title: Virtual Memory
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Virtual Memory

Type: CONCEPT

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- Optimization of Virtual Memory Locking Limits is crucial for High-Performance GPU Inference on Linux Workstations.
- Understanding the operating system's virtual memory management is crucial.
- Model weights stored in GGUF files are mapped into the virtual address space of the process using the mmap system call.
- Inactive virtual memory pages may be written to the swap partition on disk.
- The Linux kernel restricts memory-locking operations using a resource limit designated as RLIMIT_MEMLOCK, which dictates the maximum virtual address space a process can lock into physical RAM.

## From [drive-research-cuda-graph-capture-failure-workarounds-micro03](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro03.md) (2026-06-09)
- Fragmented by accumulating slot context checkpoints over time.
