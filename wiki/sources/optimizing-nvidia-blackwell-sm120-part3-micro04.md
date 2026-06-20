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
- [ROCm deployments](/entities/rocm-deployments.md) (SYSTEM)
- [ROCR_VISIBLE_DEVICES](/entities/rocr-visible-devices.md) (TOOL)
- [OOM crash](/entities/oom-crash.md) (CONCEPT)
- [GGML_SCHED_MAX_SPLIT_INPUTS](/entities/ggml-sched-max-split-inputs.md) (CONCEPT)
- [automated parameter-fitting subsystem](/entities/automated-parameter-fitting-subsystem.md) (SYSTEM)
- [GGML_CUDA_DISABLE_GRAPHS=1](/entities/ggml-cuda-disable-graphs-1.md) (SYSTEM)
- [HIP_VISIBLE_DEVICES](/entities/hip-visible-devices.md) (TOOL)
- [VRAM](/entities/vram.md) (CONCEPT)
- [GGML_ASSERT](/entities/ggml-assert.md) (CONCEPT)
- [ggml-org/llama.cpp](/entities/ggml-org-llama-cpp.md) (PROJECT)
- [PR #22133](/entities/pr-22133.md) (PROJECT)

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
