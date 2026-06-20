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
- [Sampler Sequencing](/entities/sampler-sequencing.md) (CONCEPT)
- [Blackwell-class](/entities/blackwell-class.md) (CONCEPT)
- [Logical Batch](/entities/logical-batch.md) (CONCEPT)
- [CUDA](/entities/cuda.md) (SYSTEM)
- [GPU utilization](/entities/gpu-utilization.md) (CONCEPT)
- [Parallel Decoding](/entities/parallel-decoding.md) (CONCEPT)
- [Continuous Batching](/entities/continuous-batching.md) (CONCEPT)
- [Prompt Lookup Decoding](/entities/prompt-lookup-decoding.md) (CONCEPT)
- [Min-P](/entities/min-p.md) (CONCEPT)
- [Top-P](/entities/top-p.md) (CONCEPT)
- [KV Cache Quantization](/entities/kv-cache-quantization.md) (CONCEPT)
- [llama.cpp](/entities/llama-cpp.md) (TOOL)
- [Speculative Decoding](/entities/speculative-decoding.md) (CONCEPT)
- [Draft Model](/entities/draft-model.md) (CONCEPT)
- [time-to-first-token (TTFT)](/entities/time-to-first-token-ttft.md) (CONCEPT)
- [ggml graph](/entities/ggml-graph.md) (SYSTEM)
- [Retrieval-Augmented Generation (RAG)](/entities/retrieval-augmented-generation-rag.md) (CONCEPT)
- [Quantized Johnson-Lindenstrauss (QJL)](/entities/quantized-johnson-lindenstrauss-qjl.md) (CONCEPT)
- [DGX Spark GB10](/entities/dgx-spark-gb10.md) (SYSTEM)
- [Flash Attention](/entities/flash-attention.md) (CONCEPT)
- [TurboQuant](/entities/turboquant.md) (CONCEPT)
- [Top-K](/entities/top-k.md) (CONCEPT)
- [Physical Batch](/entities/physical-batch.md) (CONCEPT)
- [VRAM](/entities/vram.md) (CONCEPT)

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
