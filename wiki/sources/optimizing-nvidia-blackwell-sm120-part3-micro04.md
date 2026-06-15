---
type: source
title: optimizing-nvidia-blackwell-sm120-part3-micro04
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# optimizing-nvidia-blackwell-sm120-part3-micro04

Ingested source summary (2026-06-09).

## Entities
- [[rocm-deployments|ROCm deployments]] (SYSTEM)
- [[rocr-visible-devices|ROCR_VISIBLE_DEVICES]] (TOOL)
- [[oom-crash|OOM crash]] (CONCEPT)
- [[ggml-sched-max-split-inputs|GGML_SCHED_MAX_SPLIT_INPUTS]] (CONCEPT)
- [[automated-parameter-fitting-subsystem|automated parameter-fitting subsystem]] (SYSTEM)
- [[ggml-cuda-disable-graphs-1|GGML_CUDA_DISABLE_GRAPHS=1]] (SYSTEM)
- [[hip-visible-devices|HIP_VISIBLE_DEVICES]] (TOOL)
- [[vram|VRAM]] (CONCEPT)
- [[ggml-assert|GGML_ASSERT]] (CONCEPT)
- [[ggml-org-llama-cpp|ggml-org/llama.cpp]] (PROJECT)
- [[pr-22133|PR #22133]] (PROJECT)

## Relations
- VRAM → RELATED_TO → OOM crash
- GGML_CUDA_DISABLE_GRAPHS=1 → RELATED_TO → VRAM
- GGML_CUDA_DISABLE_GRAPHS=1 → RELATED_TO → OOM crash
- GGML_SCHED_MAX_SPLIT_INPUTS → RELATED_TO → GGML_ASSERT
- PR #22133 → RELATED_TO → GGML_SCHED_MAX_SPLIT_INPUTS
- automated parameter-fitting subsystem → RELATED_TO → GGML_ASSERT
- automated parameter-fitting subsystem → USES → VRAM
- ROCm deployments → USES → ROCR_VISIBLE_DEVICES
- ROCm deployments → USES → HIP_VISIBLE_DEVICES
- ROCR_VISIBLE_DEVICES → RELATED_TO → HIP_VISIBLE_DEVICES
- ggml-org/llama.cpp → PART_OF → PR #22133
