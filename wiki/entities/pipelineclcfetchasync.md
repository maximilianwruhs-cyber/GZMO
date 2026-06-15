---
type: entity
title: PipelineCLCFetchAsync
created: 2026-06-08
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# PipelineCLCFetchAsync

Type: TOOL

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- A class implemented in CUTLASS to manage CLC.
- Decouples the scheduler warp from MMA and epilogue warps.
- Implements a depth of 3.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro03|optimizing-nvidia-blackwell-sm120-part2-micro03]] (2026-06-10)
- Implemented in CUTLASS
- Decouples scheduler warp from MMA and epilogue warps
- Implements a pipeline depth of 3
