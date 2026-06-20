---
type: entity
title: cudaHostRegisterPortable
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# cudaHostRegisterPortable

Type: CONCEPT

## From [drive-research-cuda-memory-locking-limits-configuration](/entities/drive-research-cuda-memory-locking-limits-configuration.md) (2026-06-08)
- A flag used by ggml_backend_cuda_register_host_buffer when invoking cudaHostRegister.
- Guarantees that the model weights are page-locked, read-only, and visible across all active CUDA contexts.
