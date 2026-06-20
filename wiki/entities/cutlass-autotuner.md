---
type: entity
title: CUTLASS autotuner
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CUTLASS autotuner

Type: TOOL

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- Falls back to slow, non-TMA templates when compute_120a causes runtime errors on SM120.
- Executes fastest TMA Warp-Specialized grouped GEMM tactics when compiled under compute_120f.
- A library where GDC execution is controlled by the #ifndef CUTLASS_GDC_ENABLED preprocessor guard.
- Internal headers evaluate the GDC enablement check as false when compiling for SM120 using generic targets.
- Requires the -DCUTLASS_GDC_ENABLED=1 flag to force the generation of physical GDC instructions.
