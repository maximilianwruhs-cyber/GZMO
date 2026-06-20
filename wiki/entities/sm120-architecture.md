---
type: entity
title: SM120 architecture
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# SM120 architecture

Type: SYSTEM

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Represents workstation-class Blackwell GPUs.
- Includes RTX PRO 6000 Blackwell Workstation Edition and RTX 5090.
- Constrained to a maximum of 99 KiB of shared memory per SM.
- Lacks physical TMEM allocations.
- Must fall back to standard Warp-Level Matrix Multiply-Accumulate (WMMA) 16x16x16 tensor core operations.
- Equipped with a GDDR7 memory bus.
