---
type: entity
title: compute_120f
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# compute_120f

Type: CONCEPT

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- A distinct target for SM12x generation in CUDA 13.0 and newer.
- Enables common architecture-specific features across the SM12x GPU family.
- Resolves initialization bugs in the CUTLASS autotuner.

## From [[drive-research-flashinfer-moe-fp4-jit-error|drive-research-flashinfer-moe-fp4-jit-error]] (2026-06-08)
- A CUDA compilation target flag
- Corresponds to the Blackwell workstation family target
- Enables optimization for workstation Blackwell cards

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- A family-specific compiler target introduced alongside CUDA 13.0.
- Acts as the optimal pathway for desktop and workstation Blackwell silicon (SM120, SM121).
- Enables a unified subset of architectural features common across all Blackwell devices.
- Provides necessary mathematical instructions while preserving complete hardware compatibility on SM120 and SM121 devices.
- When patched with GDC and Alignment, enables fast TMA WS grouped GEMM tactics.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro04|optimizing-nvidia-blackwell-sm120-part1-micro04]] (2026-06-09)
- Compilation flag (CUDA 13.0+).
- Unlocks full feature set and fast TMA WS tactics (39.0 tok/s).
- Requires a clean compilation environment and specific JIT configurations.
