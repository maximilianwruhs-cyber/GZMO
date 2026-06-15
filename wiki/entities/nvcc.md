---
type: entity
title: NVCC
created: 2026-06-08
updated: 2026-06-10
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# NVCC

Type: TOOL

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- Compiler pipeline for CUDA.
- Requires navigating subtle differences in NVCC compiler flags and code-generation targets for consumer Blackwell GPUs.

## From [[drive-research-imagine-creating-sm120-according-to-our-progress|drive-research-imagine-creating-sm120-according-to-our-progress]] (2026-06-08)
- If compiling in a Windows host environment, the NVCC compiler can trigger preprocessor failures (Error C1189) within the CUDA standard library headers (cccl) if MSVC's legacy preprocessor is engaged.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro01|optimizing-nvidia-blackwell-sm120-part1-micro01]] (2026-06-10)
- A compiler used to generate kernels for NVIDIA CUDA architectures.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro07|optimizing-nvidia-blackwell-sm120-part1-micro07]] (2026-06-10)
- The NVIDIA CUDA compiler.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro02|optimizing-nvidia-blackwell-sm120-part2-micro02]] (2026-06-10)
- Compiler used for generating code for SM12x generation.
- Introduced compute_120, compute_120a, and compute_120f targets in CUDA 13.0.
