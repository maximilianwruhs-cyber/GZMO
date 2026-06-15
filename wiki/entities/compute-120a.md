---
type: entity
title: compute_120a
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# compute_120a

Type: CONCEPT

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- A distinct target for SM12x generation in CUDA 13.0.
- Designed for architecture-specific compilation.
- Causes CUTLASS TMA Warp-Specialized grouped GEMM tactics to fail at runtime.

## From [[drive-research-flashinfer-moe-fp4-jit-error|drive-research-flashinfer-moe-fp4-jit-error]] (2026-06-08)
- A CUDA compilation target
- Attempts to leverage workstation-specific MMA instructions
- Causes segmentation fault at runtime on workstation GPUs

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- An architecture-specific compiler target under CUDA 13.x.
- Assumes the target GPU contains the complete, unabridged set of physical datacenter-class instructions.
- Causes severe runtime errors on consumer SM120 hardware.
- When unpatched, leads to NaNs in SGLang and fallback to slow non-TMA tactics in vLLM.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro04|optimizing-nvidia-blackwell-sm120-part1-micro04]] (2026-06-09)
- Compilation flag (CUDA 12.8+).
- MMA instructions are enabled but TMA WS tactics fail to initialize.
- Forces autotuner to fall back to slow non-TMA tactics (14.6 tok/s).
