---
type: entity
title: NVIDIA CUDA GPUs
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# NVIDIA CUDA GPUs

Type: SYSTEM

## From [architectural-strategy-for-stealthy-portable-cli-a](/entities/architectural-strategy-for-stealthy-portable-cli-a.md) (2026-06-08)
- Specialized hardware accelerator.
- Discovery involves querying operating system's configuration registries or virtual file systems.
- Can be detected by dynamically loading nvcuda.dll on Windows or libcuda.so on Linux.
