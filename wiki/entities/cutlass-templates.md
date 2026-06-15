---
type: entity
title: CUTLASS templates
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# CUTLASS templates

Type: TOOL

## From [[drive-research-imagine-creating-sm120-according-to-our-progress|drive-research-imagine-creating-sm120-according-to-our-progress]] (2026-06-08)
- Triggers an 'Arch conditional MMA' compile error within CUTLASS templates.
- Developers must manually patch critical synchronization and memory alignment bugs across CUTLASS.
- Internal CUTLASS guards evaluate the GDC enablement check as false.
- Modify include/cutlass/gemm/collective/sm120_blockscaled_mma_tma.hpp.
- Modify include/cutlass/epilogue/collective/sm90_epilogue_tma_warpspecialized.hpp.

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- Native FP4/FP6 CUTLASS templates fail compilation under the compute_120 flag.
- GDC execution is controlled by the #ifndef CUTLASS_GDC_ENABLED preprocessor guard.
- Employ NVIDIA's CuTe Domain-Specific Language (DSL) to partition shared memory arrays.
- Standard SM120 template mainloops often lack explicit alignment constraints for TMA parameter structures.
