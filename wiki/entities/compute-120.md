---
type: entity
title: compute_120
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# compute_120

Type: CONCEPT

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- A distinct target for SM12x generation in CUDA 13.0.
- Restricts the compiler from emitting architecture-specific instructions.
- Causes CUTLASS low-precision templates to fail compilation.

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- A baseline compiler target under CUDA 13.x.
- Restricts compilation to a generic ISA that excludes low-precision conversion and block-scaled instructions.
- Results in compilation failures when attempting to compile native FP4/FP6 CUTLASS templates.
- Does not define the macro __CUDA_ARCH_FEAT_SM120_ALL, causing GDC enablement check to evaluate as false.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro04|optimizing-nvidia-blackwell-sm120-part1-micro04]] (2026-06-09)
- Compilation flag (CUDA 12.8+).
- MMA instructions are not enabled, triggering an 'Arch conditional MMA' compile error.
