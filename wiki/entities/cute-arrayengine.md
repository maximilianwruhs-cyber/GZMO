---
type: entity
title: cute::ArrayEngine
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# cute::ArrayEngine

Type: SYSTEM

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- An unaligned type used for shared memory scale factor arrays (smem_SFA, smem_SFB) in some CUTLASS mainloops.
- Renders these arrays vulnerable to metadata loss during CuTe layout partitioning.
