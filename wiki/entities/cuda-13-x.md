---
type: entity
title: CUDA 13.x
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# CUDA 13.x

Type: TOOL

## From [drive-research-imagine-creating-sm120-according-to-our-progress](/entities/drive-research-imagine-creating-sm120-according-to-our-progress.md) (2026-06-08)
- Compiling optimized kernels under CUDA 13.x requires a precise understanding of target suffix overrides.
- CUDA graph capture pressure triggers unrecoverable Kernel-Power 41 errors.
- CUDA 13.1+ toolchain upgrades are necessary for native sub-byte execution.

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- Introduces three distinct compiler targets: compute_120, compute_120a/compute_121a, and compute_120f.
- Exhibits performance regressions across several core computational libraries.
- Update 2 was required to address critical correctness errors, execution hangs, and illegal memory accesses.

## From [optimizing-nvidia-blackwell-sm120-part1-micro06](/entities/optimizing-nvidia-blackwell-sm120-part1-micro06.md) (2026-06-10)
- Introduces three distinct compiler targets: compute_120, compute_120a/compute_121a, and compute_120f.
- Includes the NVRTC (Runtime Compilation) library.
