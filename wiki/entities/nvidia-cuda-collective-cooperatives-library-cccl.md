---
type: entity
title: NVIDIA CUDA Collective Cooperatives Library (CCCL)
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# NVIDIA CUDA Collective Cooperatives Library (CCCL)

Type: TOOL

## From [drive-research-cuda-graph-capture-failure-workarounds-micro02](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro02.md) (2026-06-09)
- Version >3.2 is required for ggml_top_k kernel execution with large tensor dimensions.
- Used in conjunction with GGML_CUDA_USE_CUB.
- Required (version >3.2) to prevent illegal memory access when executing ggml_top_k with large tensor dimensions.

## From [optimizing-nvidia-blackwell-sm120-part3-micro05](/entities/optimizing-nvidia-blackwell-sm120-part3-micro05.md) (2026-06-09)
- Version >3.2 can mitigate illegal memory access in ggml_top_k.
- Used with GGML_CUDA_USE_CUB enabled.
- Related to parallel reduction operations.

## From [optimizing-nvidia-blackwell-sm120-part3-micro06](/entities/optimizing-nvidia-blackwell-sm120-part3-micro06.md) (2026-06-09)
- It is a backend for llama.cpp.
- It is used in conjunction with NVIDIA hardware.
- It is involved in memory access and graph operations.
- It is used for compiling the backend.
- Version 3.2 or greater is recommended.
- It integrates vendor-validated parallel algorithms.
- It is an option to be enabled when compiling the backend with CCCL.
- It integrates robust, vendor-validated parallel algorithms.
