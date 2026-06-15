---
type: entity
title: CUDA Runtime API
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CUDA Runtime API

Type: SYSTEM

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- NVIDIA CUDA architecture accelerates host-to-device transfers using page-locked (pinned) host memory.
- Standard memory allocated via malloc or anonymous mmap is pageable and requires the CUDA driver to copy data into an internal, page-locked staging buffer before performing a Direct Memory Access (DMA) transfer to the GPU.
- CUDA Runtime API call cudaHostRegister registers an existing host virtual address range as page-locked memory.
- The NVIDIA driver locks these pages in physical memory and maps them directly into the GPU’s page tables.
- Historically, significant confusion arose among developers regarding whether driver-level host memory allocations were bound by the RLIMIT_MEMLOCK parameter.
- The application registers an existing host virtual address range using the CUDA Runtime API call cudaHostRegister.
- cudaHostRegister is part of the CUDA Runtime API.
