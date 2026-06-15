---
type: source
title: drive-research-llamacpp-optimization-blueprint-micro03
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-llamacpp-optimization-blueprint-micro03

Ingested source summary (2026-06-09).

## Entities
- [[sampler-sequencing|Sampler Sequencing]] (CONCEPT)
- [[blackwell-class|Blackwell-class]] (CONCEPT)
- [[logical-batch|Logical Batch]] (CONCEPT)
- [[cuda|CUDA]] (SYSTEM)
- [[gpu-utilization|GPU utilization]] (CONCEPT)
- [[parallel-decoding|Parallel Decoding]] (CONCEPT)
- [[continuous-batching|Continuous Batching]] (CONCEPT)
- [[prompt-lookup-decoding|Prompt Lookup Decoding]] (CONCEPT)
- [[min-p|Min-P]] (CONCEPT)
- [[top-p|Top-P]] (CONCEPT)
- [[kv-cache-quantization|KV Cache Quantization]] (CONCEPT)
- [[llama-cpp|llama.cpp]] (TOOL)
- [[speculative-decoding|Speculative Decoding]] (CONCEPT)
- [[draft-model|Draft Model]] (CONCEPT)
- [[time-to-first-token-ttft|time-to-first-token (TTFT)]] (CONCEPT)
- [[ggml-graph|ggml graph]] (SYSTEM)
- [[retrieval-augmented-generation-rag|Retrieval-Augmented Generation (RAG)]] (CONCEPT)
- [[quantized-johnson-lindenstrauss-qjl|Quantized Johnson-Lindenstrauss (QJL)]] (CONCEPT)
- [[dgx-spark-gb10|DGX Spark GB10]] (SYSTEM)
- [[flash-attention|Flash Attention]] (CONCEPT)
- [[turboquant|TurboQuant]] (CONCEPT)
- [[top-k|Top-K]] (CONCEPT)
- [[physical-batch|Physical Batch]] (CONCEPT)
- [[vram|VRAM]] (CONCEPT)

## Relations
- Logical Batch → RELATED_TO → Physical Batch
- Continuous Batching → USES → llama.cpp
- Parallel Decoding → RELATED_TO → KV Cache Quantization
- Speculative Decoding → USES → Draft Model
- Speculative Decoding → RELATED_TO → VRAM
- Draft Model → RELATED_TO → Speculative Decoding
- Prompt Lookup Decoding → RELATED_TO → VRAM
- Prompt Lookup Decoding → RELATED_TO → Retrieval-Augmented Generation (RAG)
- KV Cache Quantization → RELATED_TO → VRAM
- KV Cache Quantization → USES → DGX Spark GB10
- TurboQuant → USES → llama.cpp
- TurboQuant → USES → CUDA
- Quantized Johnson-Lindenstrauss (QJL) → RELATED_TO → Quantization
- Min-P → RELATED_TO → Top-K
- Min-P → RELATED_TO → Top-P
- Min-P → USES → Sampler Sequencing
- Top-K → RELATED_TO → Min-P
- Top-P → RELATED_TO → Min-P
- Sampler Sequencing → USES → Min-P
- Sampler Sequencing → USES → llama.cpp
- Physical Batch → PART_OF → ggml graph
- Logical Batch → RELATED_TO → CUDA
- DGX Spark GB10 → PART_OF → Blackwell-class
