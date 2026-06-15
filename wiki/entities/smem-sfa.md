---
type: entity
title: smem_SFA
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# smem_SFA

Type: SYSTEM

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- A shared memory scale factor array declared using the unaligned cute::ArrayEngine type in sm120_blockscaled_mma_tma.hpp.
- Vulnerable to metadata loss during CuTe layout partitioning.
