---
type: entity
title: Flash Attention (-fa)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Flash Attention (-fa)

Type: CONCEPT

## From [[drive-research-llamacpp-optimization-blueprint-micro02|drive-research-llamacpp-optimization-blueprint-micro02]] (2026-06-09)
- A revolutionary algorithm that heavily reduces the VRAM footprint of the Key-Value (KV) cache at high context sizes.
- Natively, the CUDA implementation in llama.cpp only supports symmetric configurations.
- Enabling the GGML_CUDA_FA_ALL_QUANTS=ON flag forces Flash Attention support across all possible combinations of KV cache quantization types.
