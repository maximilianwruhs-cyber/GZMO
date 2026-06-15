---
type: entity
title: GGML_CUDA_USE_CUB
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# GGML_CUDA_USE_CUB

Type: CONCEPT

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro02|drive-research-cuda-graph-capture-failure-workarounds-micro02]] (2026-06-09)
- Enabled to prevent illegal memory access when executing ggml_top_k with large tensor dimensions.
- Used in conjunction with CCCL version >3.2.

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro03|drive-research-cuda-graph-capture-failure-workarounds-micro03]] (2026-06-09)
- Enabled when compiling the backend with CCCL.
- Integrates robust, vendor-validated parallel algorithms.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro05|optimizing-nvidia-blackwell-sm120-part3-micro05]] (2026-06-09)
- Enabled to mitigate illegal memory access in ggml_top_k.
- Used in conjunction with CCCL version >3.2.
- Related to parallel reduction operations.
