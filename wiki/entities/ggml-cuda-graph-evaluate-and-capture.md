---
type: entity
title: ggml_cuda_graph_evaluate_and_capture
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# ggml_cuda_graph_evaluate_and_capture

Type: TOOL

## From [drive-research-cuda-graph-capture-failure-workarounds-micro02](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro02.md) (2026-06-09)
- Out-of-memory error occurs inside this function during MoE architectures.
- Verifies node buffer type during graph capture.
- Encounters a pointer assertion when parameters spill over to host memory.

## From [optimizing-nvidia-blackwell-sm120-part3-micro05](/entities/optimizing-nvidia-blackwell-sm120-part3-micro05.md) (2026-06-09)
- Function where OOM errors occur in MoE architectures.
- Verifies node buffer type during graph capture.
- Assertion fails when nodes are mapped to host-spillover memory.
