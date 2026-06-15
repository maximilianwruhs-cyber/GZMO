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
- [[llama-bench|llama-bench]] (TOOL)
- [[nvidia-cuda-collective-cooperatives-library-cccl|NVIDIA CUDA Collective Cooperatives Library (CCCL)]] (SYSTEM)
- [[multi-gpu|multi-GPU]] (CONCEPT)
- [[llama-cpp|llama.cpp]] (PROJECT)
- [[nvidia-blackwell-sm120|NVIDIA Blackwell SM120]] (SYSTEM)
- [[tensor-split|tensor-split]] (CONCEPT)
- [[kv-cache|KV-Cache]] (CONCEPT)
- [[systemd|systemd]] (TOOL)
- [[ggml|GGML]] (SYSTEM)

## Relations
- llama-bench → PART_OF → llama.cpp
- llama.cpp → USES → GGML
- NVIDIA Blackwell SM120 → RELATED_TO → multi-GPU
- multi-GPU → RELATED_TO → tensor-split
- systemd → RELATED_TO → multi-GPU
- llama-bench → RELATED_TO → KV-Cache
- llama-bench → RELATED_TO → multi-GPU
