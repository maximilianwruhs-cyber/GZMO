---
type: entity
title: GGUF model scales
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# GGUF model scales

Type: CONCEPT

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- The table details calculated minimum memlock limits in KiB for common GGUF model scales and quantization levels.
- Model weights are stored in GGUF files.
- Model weights stored in GGUF files are mapped into the virtual address space of the process using the mmap system call.
- A memory-mapped GGUF model buffer can be registered as page-locked memory using cudaHostRegister.
