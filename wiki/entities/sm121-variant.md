---
type: entity
title: SM121 variant
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# SM121 variant

Type: SYSTEM

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Represents workstation-class Blackwell GPUs.
- Includes DGX Spark GB10.
- Constrained to a maximum of 99 KiB of shared memory per SM.
- Lacks physical TMEM allocations.
- Must fall back to standard Warp-Level Matrix Multiply-Accumulate (WMMA) 16x16x16 tensor core operations.
