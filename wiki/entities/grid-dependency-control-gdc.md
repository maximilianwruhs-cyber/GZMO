---
type: entity
title: Grid Dependency Control (GDC)
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# Grid Dependency Control (GDC)

Type: CONCEPT

## From [drive-research-blackwell-sm120-gemm-optimization-guide](/entities/drive-research-blackwell-sm120-gemm-optimization-guide.md) (2026-06-08)
- Barriers that prevent data hazards and desynchronization between cooperative thread blocks.
- CUTLASS uses a compile-time preprocessor check to guard GDC code generation.
- Explicit enablement flag (-DCUTLASS_GDC_ENABLED=1) is required for numerical correctness.

## From [drive-research-flashinfer-moe-fp4-jit-error](/entities/drive-research-flashinfer-moe-fp4-jit-error.md) (2026-06-08)
- Synchronization barriers required by Blackwell devices
- Not compiled as active PTX instructions when GDC is not enabled
- Enabled by compute_120f target

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- Instructions managed on the device to synchronize Grid Dependency Control.
- Includes griddepcontrol.wait and griddepcontrol.launch_dependents.
- Execution is controlled by the #ifndef CUTLASS_GDC_ENABLED preprocessor guard within CUTLASS.
- When not enabled, GDC barriers compile as empty no-ops, leading to synchronization breakdown.

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Barrier bug exists in CUTLASS compilation pipeline.
- Compiler flags for GDC are silently ignored on standard SM120 environments.
- GDC barriers compile as no-ops on SM120, causing missing synchronization.
