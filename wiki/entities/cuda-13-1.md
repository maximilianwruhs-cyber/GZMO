---
type: entity
title: CUDA 13.1
created: 2026-06-08
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# CUDA 13.1

Type: SYSTEM

## From [drive-research-flashinfer-moe-fp4-jit-error](/entities/drive-research-flashinfer-moe-fp4-jit-error.md) (2026-06-08)
- Environment where JIT compilation errors occur
- Introduced compute_120f target flag resolving issues

## From [optimizing-nvidia-blackwell-sm120-part1-micro06](/entities/optimizing-nvidia-blackwell-sm120-part1-micro06.md) (2026-06-10)
- Runtime compilation environment used for low-precision backends.
- Introduces specific compiler targets like compute_120f.
- Requires specific updates to address correctness errors in cuBLASLtMatmul().
