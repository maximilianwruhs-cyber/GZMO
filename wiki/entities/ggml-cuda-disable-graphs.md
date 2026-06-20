---
type: entity
title: GGML_CUDA_DISABLE_GRAPHS
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# GGML_CUDA_DISABLE_GRAPHS

Type: SYSTEM

## From [drive-research-llamacpp-gpu-memory-reporting-bug](/entities/drive-research-llamacpp-gpu-memory-reporting-bug.md) (2026-06-08)
- An active backend queried by llama_params_fit.
- Driver frameworks can allocate a base memory footprint during enumeration.
- CUDA graphs can accumulate and cause memory leaks.
- Environment variable to disable CUDA graphs.
- Prevents memory exhaustion caused by unbound CUDA graph allocations.

## From [drive-research-cuda-graph-capture-failure-workarounds-micro03](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro03.md) (2026-06-09)
- Environment variable to set.
- Forces the backend to bypass graph capture.
- Avoids cache leaks and instantiation failures.
- Maintains long-term memory stability.
