---
type: source
title: drive-research-llm-inference-engine-audit-2026-micro01
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-llm-inference-engine-audit-2026-micro01

Ingested source summary (2026-06-10).

## Entities
- [NVIDIA](/entities/nvidia.md) (ORGANIZATION)
- [P-EAGLE](/entities/p-eagle.md) (CONCEPT)
- [RadixAttention](/entities/radixattention.md) (CONCEPT)
- [DeepSeek V4](/entities/deepseek-v4.md) (CONCEPT)
- [FlashMLA](/entities/flashmla.md) (SYSTEM)
- [Gaudi 3](/entities/gaudi-3.md) (CONCEPT)
- [Hopper](/entities/hopper.md) (CONCEPT)
- [Blackwell](/entities/blackwell.md) (CONCEPT)
- [vLLM](/entities/vllm.md) (SYSTEM)
- [Llama 4](/entities/llama-4.md) (CONCEPT)
- [TensorRT-LLM](/entities/tensorrt-llm.md) (SYSTEM)
- [Apple](/entities/apple.md) (ORGANIZATION)
- [FlashInfer](/entities/flashinfer.md) (SYSTEM)
- [LMDeploy](/entities/lmdeploy.md) (SYSTEM)
- [Intel](/entities/intel.md) (ORGANIZATION)
- [TurboMind](/entities/turbomind.md) (SYSTEM)
- [MLX](/entities/mlx.md) (SYSTEM)
- [ATOM](/entities/atom.md) (SYSTEM)
- [PagedAttention](/entities/pagedattention.md) (CONCEPT)
- [SGLang](/entities/sglang.md) (SYSTEM)
- [AMD](/entities/amd.md) (ORGANIZATION)

## Relations
- vLLM → USES → PagedAttention
- SGLang → USES → RadixAttention
- LMDeploy → USES → TurboMind
- vLLM → USES → FlashInfer
- SGLang → USES → FlashInfer
- P-EAGLE → PART_OF → vLLM
- TensorRT-LLM → RELATED_TO → NVIDIA
- ATOM → RELATED_TO → AMD
- Gaudi 3 → RELATED_TO → Intel
- MLX → RELATED_TO → Apple
