---
type: entity
title: ggml-cuda.cu
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# ggml-cuda.cu

Type: SYSTEM

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro02|drive-research-cuda-graph-capture-failure-workarounds-micro02]] (2026-06-09)
- Assertion failure occurs inside this file during multi-GPU row-split configurations.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro05|optimizing-nvidia-blackwell-sm120-part3-micro05]] (2026-06-09)
- File where assertion failure occurs with row-splitting logic.
- Assertion `!(split && ne02 < ne12)` fails.
- Related to synchronization failures in multi-GPU row-split configurations.
