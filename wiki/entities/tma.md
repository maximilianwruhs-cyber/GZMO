---
type: entity
title: TMA
created: 2026-06-10
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# TMA

Type: SYSTEM

## From [[optimizing-nvidia-blackwell-sm120-part1-micro06|optimizing-nvidia-blackwell-sm120-part1-micro06]] (2026-06-10)
- Tensor Memory Accelerator for zero-overhead global-to-shared memory transfers.
- Requires 64-byte alignment for descriptor structures.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro07|optimizing-nvidia-blackwell-sm120-part1-micro07]] (2026-06-10)
- Requires 64-byte alignment on parameter structures (TMA_A, TMA_B, TMA_SFA, TMA_SFB, TMA_C, TMA_D).
