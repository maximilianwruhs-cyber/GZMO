---
type: source
title: optimizing-nvidia-blackwell-sm120-part3-micro06
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# optimizing-nvidia-blackwell-sm120-part3-micro06

Ingested source summary (2026-06-09).

## Entities
- [llama-bench](/entities/llama-bench.md) (TOOL)
- [NVIDIA CUDA Collective Cooperatives Library (CCCL)](/entities/nvidia-cuda-collective-cooperatives-library-cccl.md) (SYSTEM)
- [multi-GPU](/entities/multi-gpu.md) (CONCEPT)
- [llama.cpp](/entities/llama-cpp.md) (PROJECT)
- [NVIDIA Blackwell SM120](/entities/nvidia-blackwell-sm120.md) (SYSTEM)
- [tensor-split](/entities/tensor-split.md) (CONCEPT)
- [KV-Cache](/entities/kv-cache.md) (CONCEPT)
- [systemd](/entities/systemd.md) (TOOL)
- [GGML](/entities/ggml.md) (SYSTEM)

## Relations
- llama-bench → PART_OF → llama.cpp
- llama.cpp → USES → GGML
- NVIDIA Blackwell SM120 → RELATED_TO → multi-GPU
- multi-GPU → RELATED_TO → tensor-split
- systemd → RELATED_TO → multi-GPU
- llama-bench → RELATED_TO → KV-Cache
- llama-bench → RELATED_TO → multi-GPU
