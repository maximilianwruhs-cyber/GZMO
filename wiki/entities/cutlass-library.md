---
type: entity
title: CUTLASS library
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# CUTLASS library

Type: TOOL

## From [[optimizing-nvidia-blackwell-sm120-part1-micro04|optimizing-nvidia-blackwell-sm120-part1-micro04]] (2026-06-09)
- Contains critical bugs when deploying MoE models using native NVFP4 weight formats on SM120.
- Templates have a shared memory layout mismatch when compiled for SM120.
- Grouped GEMM tactics are hardcoded to assume SM100-class hardware parameters.
- Has a bug within the CuTe DSL parser targeting SM120 FP4 TMA descriptor lowering.
