---
type: source
title: drive-research-erbandbreite-und-latenzengpässe
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-erbandbreite-und-latenzengpässe

Ingested source summary (2026-06-08).

## Entities
- [[dynamic-time-warping|Dynamic Time Warping]] (TOOL)
- [[high-bandwidth-memory-hbm|High Bandwidth Memory (HBM)]] (SYSTEM)
- [[general-matrix-multiply-gemm|General Matrix Multiply (GEMM)]] (CONCEPT)
- [[sliding-window-attention-layers|sliding window attention layers]] (CONCEPT)
- [[rejection-sampling|Rejection Sampling]] (CONCEPT)
- [[qwen-2-5-vl-72b|Qwen 2.5-VL 72B]] (SYSTEM)
- [[full-attention-layers|full attention layers]] (CONCEPT)
- [[llama-3-1-70b|Llama 3.1 70B]] (SYSTEM)
- [[nvidia-a100|NVIDIA A100]] (HARDWARE)
- [[cascade|Cascade]] (CONCEPT)
- [[tensorrt-llm|TensorRT-LLM]] (TOOL)
- [[ctc-drafter|CTC-drafter]] (SYSTEM)
- [[opt-66b|OPT 66B]] (SYSTEM)
- [[pagedattention|PagedAttention]] (TOOL)
- [[code-completion|code completion]] (CONCEPT)
- [[mathematical-reasoning|mathematical reasoning]] (CONCEPT)
- [[vocabtrim|VocabTrim]] (CONCEPT)
- [[moe-spec|MoE-Spec]] (CONCEPT)
- [[draft-model-selection-strategies|Draft Model Selection Strategies]] (CONCEPT)
- [[cuda-kernels|CUDA kernels]] (TOOL)
- [[llama-3-1-8b|Llama 3.1 8B]] (SYSTEM)
- [[online-speculative-decoding|Online Speculative Decoding]] (CONCEPT)
- [[gumiho|Gumiho]] (SYSTEM)
- [[parallel-eagle-p-eagle|Parallel-EAGLE (P-EAGLE)]] (SYSTEM)
- [[medusa|Medusa]] (SYSTEM)
- [[memory-allocators|memory allocators]] (TOOL)
- [[gpu-memory|GPU memory]] (SYSTEM)
- [[mixture-of-experts-architectures|Mixture of Experts architectures]] (CONCEPT)
- [[block-verification|Block Verification]] (CONCEPT)
- [[llama-3-1-405b|Llama 3.1 405B]] (SYSTEM)
- [[target-model|target model]] (SYSTEM)
- [[speculative-sparsity-paradox|Speculative Sparsity Paradox]] (CONCEPT)
- [[distillspec|DistillSpec]] (CONCEPT)
- [[entropy-aware-speculative-decoding|Entropy-Aware Speculative Decoding]] (CONCEPT)
- [[autoregressive-text-generation|Autoregressive Text Generation]] (CONCEPT)
- [[tokentiming|TokenTiming]] (CONCEPT)
- [[memory-pools|memory pools]] (SYSTEM)
- [[inference-engine|inference engine]] (SYSTEM)
- [[key-value-kv-cache|Key-Value (KV) Cache]] (CONCEPT)
- [[vllm|vLLM]] (TOOL)
- [[knowledge-distillation-kd|Knowledge Distillation (KD)]] (CONCEPT)
- [[speckd|SpecKD]] (CONCEPT)
- [[ai-accelerators|AI Accelerators]] (SYSTEM)
- [[sglang|SGLang]] (TOOL)
- [[qwen-2-5-vl-7b|Qwen 2.5-VL 7B]] (SYSTEM)
- [[amd-mi300x|AMD MI300X]] (HARDWARE)
- [[griffin|GRIFFIN]] (CONCEPT)
- [[nvidia-h200|NVIDIA H200]] (HARDWARE)
- [[vocabulary-alignment|Vocabulary Alignment]] (CONCEPT)
- [[exspec|EXSpec]] (TOOL)
- [[llama-3-2-1b|Llama 3.2 1B]] (SYSTEM)
- [[large-language-models-llms|Large Language Models (LLMs)]] (SYSTEM)
- [[ragged-tensor-problem|Ragged Tensor Problem]] (CONCEPT)
- [[deepseek-v3|DeepSeek-V3]] (SYSTEM)
- [[batched-attention-optimized-speculative-sampling-bass|Batched Attention-optimized Speculative Sampling (BASS)]] (TOOL)
- [[omnidraft|OmniDraft]] (CONCEPT)
- [[memory-manager|memory manager]] (SYSTEM)
- [[mixtral-8x7b|Mixtral 8x7B]] (SYSTEM)
- [[mixture-of-experts-moe|Mixture of Experts (MoE)]] (CONCEPT)

## Relations
- Large Language Models (LLMs) → USES → Autoregressive Text Generation
- Autoregressive Text Generation → USES → High Bandwidth Memory (HBM)
- Autoregressive Text Generation → USES → Key-Value (KV) Cache
- AI Accelerators → USES → High Bandwidth Memory (HBM)
- TokenTiming → USES → Dynamic Time Warping
- TokenTiming → RELATED_TO → Vocabulary Alignment
- OmniDraft → RELATED_TO → Vocabulary Alignment
- VocabTrim → RELATED_TO → Vocabulary Alignment
- DistillSpec → RELATED_TO → Knowledge Distillation (KD)
- SpecKD → RELATED_TO → Knowledge Distillation (KD)
- GRIFFIN → RELATED_TO → Knowledge Distillation (KD)
- Online Speculative Decoding → RELATED_TO → Knowledge Distillation (KD)
- Medusa → PART_OF → Large Language Models (LLMs)
- Parallel-EAGLE (P-EAGLE) → PART_OF → Large Language Models (LLMs)
- Speculative Sparsity Paradox → RELATED_TO → Mixture of Experts (MoE)
- MoE-Spec → RELATED_TO → Mixture of Experts (MoE)
- Cascade → RELATED_TO → Mixture of Experts (MoE)
- vLLM → USES → PagedAttention
- SGLang → USES → PagedAttention
- TensorRT-LLM → USES → PagedAttention
- vLLM → USES → Key-Value (KV) Cache
- SGLang → USES → Key-Value (KV) Cache
- TensorRT-LLM → USES → Key-Value (KV) Cache
- inference engine → USES → memory allocators
- memory allocators → PART_OF → GPU memory
- target model → USES → memory manager
- memory manager → PART_OF → Key-Value (KV) Cache
- memory pools → PART_OF → Key-Value (KV) Cache
- memory pools → PART_OF → full attention layers
- memory pools → PART_OF → sliding window attention layers
- Ragged Tensor Problem → RELATED_TO → General Matrix Multiply (GEMM)
- Ragged Tensor Problem → RELATED_TO → Key-Value (KV) Cache
- BASS → USES → CUDA kernels
- Llama 3.1 70B → RELATED_TO → NVIDIA H200
- Llama 3.2 1B → RELATED_TO → Llama 3.1 70B
- Llama 3.1 70B → RELATED_TO → AMD MI300X
- Llama 3.1 8B → RELATED_TO → NVIDIA A100
