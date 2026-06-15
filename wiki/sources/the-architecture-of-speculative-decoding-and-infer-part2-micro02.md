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
- [[gumiho|Gumiho]] (SYSTEM)
- [[deepseek-v3|DeepSeek-V3]] (SYSTEM)
- [[parallel-eagle-p-eagle|Parallel-EAGLE (P-EAGLE)]] (SYSTEM)
- [[speculative-sparsity-paradox|Speculative Sparsity Paradox]] (CONCEPT)
- [[online-speculative-decoding|Online Speculative Decoding]] (CONCEPT)
- [[griffin|GRIFFIN]] (SYSTEM)
- [[pagedattention|PagedAttention]] (TOOL)
- [[moe-spec|MoE-Spec]] (SYSTEM)
- [[vllm|vLLM]] (SYSTEM)
- [[llama-3-3-70b|Llama 3.3 70B]] (SYSTEM)
- [[llama-3-2-1b|Llama 3.2 1B]] (SYSTEM)
- [[mixture-of-experts-moe|Mixture of Experts (MoE)]] (CONCEPT)
- [[eagle-3|EAGLE-3]] (SYSTEM)
- [[eagle-2|EAGLE-2]] (SYSTEM)
- [[sglang|SGLang]] (SYSTEM)
- [[cascade|Cascade]] (SYSTEM)
- [[large-language-models-llms|Large Language Models (LLMs)]] (SYSTEM)
- [[token-guided-fusion|Token-Guided Fusion]] (CONCEPT)
- [[exspec|EXSpec]] (SYSTEM)
- [[ctc-drafter|CTC-drafter]] (SYSTEM)
- [[medusa|Medusa]] (SYSTEM)
- [[mixtral-8x7b|Mixtral 8x7B]] (SYSTEM)
- [[llama-4|Llama 4]] (SYSTEM)
- [[llama-3-1-8b|Llama 3.1 8B]] (SYSTEM)
- [[ragged-tensor-problem|Ragged Tensor Problem]] (CONCEPT)
- [[multi-token-prediction-mtp|Multi-Token Prediction (MTP)]] (CONCEPT)
- [[tensorrt-llm|TensorRT-LLM]] (SYSTEM)
- [[batched-attention-optimized-speculative-sampling-bass|Batched Attention-optimized Speculative Sampling (BASS)]] (SYSTEM)
- [[qwen-2-5-vl-72b|Qwen 2.5-VL 72B]] (SYSTEM)
- [[qwen-2-5-vl-7b|Qwen 2.5-VL 7B]] (SYSTEM)

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
