---
type: entity
title: cudaHostRegisterReadOnly
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# cudaHostRegisterReadOnly

Type: CONCEPT

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- A flag used by ggml_backend_cuda_register_host_buffer when invoking cudaHostRegister.
- Guarantees that the model weights are page-locked, read-only, and visible across all active CUDA contexts.
