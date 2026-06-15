---
type: entity
title: Qwen3.5-27B
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Qwen3.5-27B

Type: SYSTEM

## From [[architectures-and-optimizations-for-speculative-de-micro04|architectures-and-optimizations-for-speculative-de-micro04]] (2026-06-09)
- A highly capable model.
- A Q4_K_M format requires approximately 16.5 GB footprint.

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro02|drive-research-cuda-graph-capture-failure-workarounds-micro02]] (2026-06-09)
- Models where row splits trigger an assertion failure inside ggml-cuda.cu.
- Highly vulnerable to synchronization failures in multi-GPU row-split configurations.
