---
type: entity
title: sm120_mma_tma_blockwise_scaling.hpp
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# sm120_mma_tma_blockwise_scaling.hpp

Type: BOOK

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- A working CUTLASS mainloop that uses cute::array_aligned, defaulting to a 16-byte alignment for shared memory scale factor arrays.
- A consumer/desktop workstation variant of the NVIDIA Blackwell GPU architecture (e.g., RTX 5090, RTX PRO 6000).
- Routes FP4/FP6 instructions through a different hardware mechanism than SM100, necessitating distinct compile-time flag structures.
- Typically paired with an x86_64 host system, utilizing discrete GDDR7 memory channels.
- Reports a physical compute capability of exactly 12.0.
- Suffers from silent compilation failures when compiling for GDC instructions using generic targets.
