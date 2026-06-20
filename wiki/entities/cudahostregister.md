---
type: entity
title: cudaHostRegister
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# cudaHostRegister

Type: TOOL

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- CUDA Runtime API call that registers an existing host virtual address range as page-locked memory.
- The NVIDIA driver locks these pages in physical memory and maps them directly into the GPU’s page tables.
- The application registers an existing host virtual address range—such as a memory-mapped GGUF model buffer—as page-locked memory.
- When cudaHostRegister is executed against an arbitrary block of host RAM, the underlying driver requests page pinning from the kernel virtual memory manager.
- The system call fails if the size of the memory buffer exceeds the RLIMIT_MEMLOCK limit.
- cudaHostRegister returns cudaErrorMemoryAllocation or a generic runtime error code (such as error code 30) when the system's RLIMIT_MEMLOCK is exceeded.
- cudaHostRegister can be used with flags cudaHostRegisterPortable and cudaHostRegisterReadOnly.
