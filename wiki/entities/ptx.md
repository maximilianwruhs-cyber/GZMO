---
type: entity
title: PTX
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# PTX

Type: CONCEPT

## From [[drive-research-imagine-creating-sm120-according-to-our-progress|drive-research-imagine-creating-sm120-according-to-our-progress]] (2026-06-08)
- The choice of compiler flags dictates exactly which PTX instructions are generated.
- Marlin implements dequantization (FP16 x INT4 or FP16 x FP4) entirely in software via inline PTX vector instructions on vector cores.
- Explicitly pass the GDC enablement flag directly to the compiler's command line to force the preprocessor to define the macro and emit the physical PTX instructions (griddepcontrol.wait and griddepcontrol.launch).

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- Parallel Thread Execution virtual instruction set.
- Dynamically produced by NVRTC.
- Explicit barrier controls are required for optimizing low-level kernels on Blackwell.
- The instruction prefetch.tensormap requires TMA descriptors to be strictly aligned to 64-byte boundaries.
- Physical griddepcontrol PTX instructions are compiled directly into device code when GDC is enabled.
