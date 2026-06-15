---
type: entity
title: CUTLASS
created: 2026-06-08
updated: 2026-06-10
sources: 10
tags: []
status: draft
gzmo_synthetic: true
---










# CUTLASS

Type: TOOL

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- Template library used for GEMM optimization.
- CUTLASS 4.5.0 is mentioned.
- Modern compilation pipelines, including CUTLASS 4.5.0, utilize PTX patterns.
- Requires manual software-level patches for correctness on consumer Blackwell hardware.
- Grouped GEMM
- CUTLASS Patches
- CUTLASS Documentation
- Programming Blackwell Tensor Cores with CUTLASS
- CUTLASS Tutorial
- NVIDIA/cutlass
- CUTLASS SM120 FP8 GEMM
- Blackwell Cluster Launch Control

## From [[drive-research-what-else-can-directly-be-aligned-with-our-common|drive-research-what-else-can-directly-be-aligned-with-our-common]] (2026-06-08)
- Default block-scaled tile generation tactics are hardcoded to assume a reduction dimension of K >= 128.

## From [[drive-research-marlin-baseline-for-early-deployments-micro02|drive-research-marlin-baseline-for-early-deployments-micro02]] (2026-06-09)
- Builder originally assumed K dimension was >= 128.
- Block scale factor layout was hardcoded.
- Custom K=64 tile generation templates were introduced to address constraints.

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro02|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro02]] (2026-06-09)
- Low-level C++ template structures are bypassed by cuTile.
- Patches must be applied to local CUTLASS source files.
- The runtime environment must prioritize the patched CUTLASS backend.

## From [[drive-research-marlin-baseline-for-early-deployments-micro01|drive-research-marlin-baseline-for-early-deployments-micro01]] (2026-06-10)
- Library containing grouped GEMM tactics and CuTe DSL.
- Contains bugs related to SM120 shared memory layout and GDC barriers.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro05|optimizing-nvidia-blackwell-sm120-part1-micro05]] (2026-06-10)
- Builder that originally assumed K dimension >= 128
- Grouped templates can cause issues on SM120

## From [[optimizing-nvidia-blackwell-sm120-part1-micro06|optimizing-nvidia-blackwell-sm120-part1-micro06]] (2026-06-10)
- Template library used for matrix multiplication.
- Uses GDC (Grid Dependency Control) instructions for synchronization.
- Employs CuTe DSL for partitioning shared memory arrays.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro07|optimizing-nvidia-blackwell-sm120-part1-micro07]] (2026-06-10)
- Requires patches to local source files to enforce hardware alignment.
- Used as a backend for dense layers via the VLLM_NVFP4_GEMM_BACKEND environment variable.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro02|optimizing-nvidia-blackwell-sm120-part2-micro02]] (2026-06-10)
- Template library used for achieving peak GEMM performance.
- Version 4.5.0 requires manual patches for SM120/SM121 correctness.
- Uses PTX patterns to build mixed-precision operations.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro03|optimizing-nvidia-blackwell-sm120-part2-micro03]] (2026-06-10)
- Version 4.5.0 includes specific tile sizes for SM120
- Implements PipelineCLCFetchAsync class
- Provides documentation for Blackwell functionality
