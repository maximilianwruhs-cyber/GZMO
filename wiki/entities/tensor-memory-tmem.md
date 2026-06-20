---
type: entity
title: Tensor Memory (TMEM)
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Tensor Memory (TMEM)

Type: SYSTEM

## From [drive-research-blackwell-sm120-gemm-optimization-guide](/entities/drive-research-blackwell-sm120-gemm-optimization-guide.md) (2026-06-08)
- SM100 architecture introduces 256 KB of TMEM per SM.
- TMEM acts as a dedicated on-chip SRAM structure.
- Consumer-grade Blackwell GPUs contain no TMEM hardware.

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Is a specialized, low-latency accumulator register space.
- SM100 has 256 KiB per warp.
- SM120 and SM121 lack physical TMEM allocations.
