---
type: source
title: the-architecture-of-speculative-decoding-and-infer-part2-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# the-architecture-of-speculative-decoding-and-infer-part2-micro02

Ingested source summary (2026-06-09).

## Entities
- [Gumiho](/entities/gumiho.md) (SYSTEM)
- [DeepSeek-V3](/entities/deepseek-v3.md) (SYSTEM)
- [Parallel-EAGLE (P-EAGLE)](/entities/parallel-eagle-p-eagle.md) (SYSTEM)
- [Speculative Sparsity Paradox](/entities/speculative-sparsity-paradox.md) (CONCEPT)
- [Online Speculative Decoding](/entities/online-speculative-decoding.md) (CONCEPT)
- [GRIFFIN](/entities/griffin.md) (SYSTEM)
- [PagedAttention](/entities/pagedattention.md) (TOOL)
- [MoE-Spec](/entities/moe-spec.md) (SYSTEM)
- [vLLM](/entities/vllm.md) (SYSTEM)
- [Llama 3.3 70B](/entities/llama-3-3-70b.md) (SYSTEM)
- [Llama 3.2 1B](/entities/llama-3-2-1b.md) (SYSTEM)
- [Mixture of Experts (MoE)](/entities/mixture-of-experts-moe.md) (CONCEPT)
- [EAGLE-3](/entities/eagle-3.md) (SYSTEM)
- [EAGLE-2](/entities/eagle-2.md) (SYSTEM)
- [SGLang](/entities/sglang.md) (SYSTEM)
- [Cascade](/entities/cascade.md) (SYSTEM)
- [Large Language Models (LLMs)](/entities/large-language-models-llms.md) (SYSTEM)
- [Token-Guided Fusion](/entities/token-guided-fusion.md) (CONCEPT)
- [EXSpec](/entities/exspec.md) (SYSTEM)
- [CTC-drafter](/entities/ctc-drafter.md) (SYSTEM)
- [Medusa](/entities/medusa.md) (SYSTEM)
- [Mixtral 8x7B](/entities/mixtral-8x7b.md) (SYSTEM)
- [Llama 4](/entities/llama-4.md) (SYSTEM)
- [Llama 3.1 8B](/entities/llama-3-1-8b.md) (SYSTEM)
- [Ragged Tensor Problem](/entities/ragged-tensor-problem.md) (CONCEPT)
- [Multi-Token Prediction (MTP)](/entities/multi-token-prediction-mtp.md) (CONCEPT)
- [TensorRT-LLM](/entities/tensorrt-llm.md) (SYSTEM)
- [Batched Attention-optimized Speculative Sampling (BASS)](/entities/batched-attention-optimized-speculative-sampling-bass.md) (SYSTEM)
- [Qwen 2.5-VL 72B](/entities/qwen-2-5-vl-72b.md) (SYSTEM)
- [Qwen 2.5-VL 7B](/entities/qwen-2-5-vl-7b.md) (SYSTEM)

## Relations
- GRIFFIN → USES → Token-Guided Fusion
- Gumiho → PART_OF → Large Language Models (LLMs)
- DeepSeek-V3 → USES → Multi-Token Prediction (MTP)
- DeepSeek-V3 → RELATED_TO → Mixture of Experts (MoE)
- Mixtral 8x7B → PART_OF → Mixture of Experts (MoE)
- Llama 4 → PART_OF → Mixture of Experts (MoE)
- MoE-Spec → RELATED_TO → Mixture of Experts (MoE)
- Cascade → RELATED_TO → Mixture of Experts (MoE)
- vLLM → USES → PagedAttention
- SGLang → USES → PagedAttention
- TensorRT-LLM → USES → PagedAttention
- EXSpec → RELATED_TO → Ragged Tensor Problem
- Batched Attention-optimized Speculative Sampling (BASS) → RELATED_TO → Ragged Tensor Problem
- Llama 3.3 70B → RELATED_TO → Llama 3.2 1B
- Llama 3.1 8B → RELATED_TO → EAGLE-3
- Qwen 2.5-VL 72B → RELATED_TO → Qwen 2.5-VL 7B
