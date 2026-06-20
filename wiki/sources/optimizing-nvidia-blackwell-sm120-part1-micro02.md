---
type: source
title: optimizing-nvidia-blackwell-sm120-part1-micro02
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# optimizing-nvidia-blackwell-sm120-part1-micro02

Ingested source summary (2026-06-09).

## Entities
- [--draft-p-min](/entities/draft-p-min.md) (CONCEPT)
- [Top-K samplers](/entities/top-k-samplers.md) (CONCEPT)
- [Speculative Decoding](/entities/speculative-decoding.md) (CONCEPT)
- [n-grams](/entities/n-grams.md) (CONCEPT)
- [confidence-scaled thresholding](/entities/confidence-scaled-thresholding.md) (CONCEPT)
- [turbo4](/entities/turbo4.md) (TOOL)
- [Top-P samplers](/entities/top-p-samplers.md) (TOOL)
- [--sampler-seq](/entities/sampler-seq.md) (TOOL)
- [llama.cpp CUDA backend](/entities/llama-cpp-cuda-backend.md) (SYSTEM)
- [logical batching](/entities/logical-batching.md) (CONCEPT)
- [KV Cache Quantization](/entities/kv-cache-quantization.md) (CONCEPT)
- [optimizing-nvidia-blackwell-sm120-part1](/entities/optimizing-nvidia-blackwell-sm120-part1.md) (PROJECT)
- [target model](/entities/target-model.md) (CONCEPT)
- [Sampling Optimization](/entities/sampling-optimization.md) (CONCEPT)
- [Key-Value cache](/entities/key-value-cache.md) (CONCEPT)
- [--draft-max](/entities/draft-max.md) (TOOL)
- [GPU](/entities/gpu.md) (SYSTEM)
- [attention mechanics](/entities/attention-mechanics.md) (CONCEPT)
- [Quantized Johnson-Lindenstrauss (QJL)](/entities/quantized-johnson-lindenstrauss-qjl.md) (CONCEPT)
- [logical batch size](/entities/logical-batch-size.md) (CONCEPT)
- [physical micro-batch size](/entities/physical-micro-batch-size.md) (CONCEPT)
- [probability shaping](/entities/probability-shaping.md) (CONCEPT)
- [--ctv q8_0](/entities/ctv-q8-0.md) (TOOL)
- [Retrieval-Augmented Generation (RAG)](/entities/retrieval-augmented-generation-rag.md) (CONCEPT)
- [document summarization](/entities/document-summarization.md) (CONCEPT)
- [coding tasks](/entities/coding-tasks.md) (CONCEPT)
- [temperature scaling](/entities/temperature-scaling.md) (CONCEPT)
- [f16](/entities/f16.md) (CONCEPT)
- [Mean Squared Error (MSE)](/entities/mean-squared-error-mse.md) (CONCEPT)
- [ggml graph](/entities/ggml-graph.md) (SYSTEM)
- [-ub (Physical Batch)](/entities/ub-physical-batch.md) (TOOL)
- [-cb (continuous batching)](/entities/cb-continuous-batching.md) (TOOL)
- [TurboQuant](/entities/turboquant.md) (CONCEPT)
- [q4_0](/entities/q4-0.md) (CONCEPT)
- [Prompt Lookup Decoding](/entities/prompt-lookup-decoding.md) (CONCEPT)
- [Blackwell-class DGX Spark GB10](/entities/blackwell-class-dgx-spark-gb10.md) (SYSTEM)
- [Lloyd-Max scalar quantization](/entities/lloyd-max-scalar-quantization.md) (CONCEPT)
- [Top-P](/entities/top-p.md) (CONCEPT)
- [parallel decoding](/entities/parallel-decoding.md) (CONCEPT)
- [-c (Context)](/entities/c-context.md) (TOOL)
- [physical memory limits](/entities/physical-memory-limits.md) (CONCEPT)
- [Logit Shaping](/entities/logit-shaping.md) (CONCEPT)
- [CUDA cores](/entities/cuda-cores.md) (SYSTEM)
- [-fa (Flash Attention)](/entities/fa-flash-attention.md) (TOOL)
- [-md /path/to/draft-model.gguf](/entities/md-path-to-draft-model-gguf.md) (TOOL)
- [--draft-p-split](/entities/draft-p-split.md) (TOOL)
- [--temp](/entities/temp.md) (TOOL)
- [turbo3](/entities/turbo3.md) (TOOL)
- [-lcs (lookup-cache-static)](/entities/lcs-lookup-cache-static.md) (TOOL)
- [--ctk q8_0](/entities/ctk-q8-0.md) (TOOL)
- [VRAM](/entities/vram.md) (CONCEPT)
- [-lcd (lookup-cache-dynamic)](/entities/lcd-lookup-cache-dynamic.md) (TOOL)
- [batching layer](/entities/batching-layer.md) (CONCEPT)
- [-np (number of parallel sequences)](/entities/np-number-of-parallel-sequences.md) (TOOL)
- [RTX 3090](/entities/rtx-3090.md) (SYSTEM)
- [-b (Logical Batch)](/entities/b-logical-batch.md) (TOOL)

## Relations
- -b (Logical Batch) → RELATED_TO → logical batching
- -ub (Physical Batch) → RELATED_TO → physical micro-batch size
- -ub (Physical Batch) → RELATED_TO → -b (Logical Batch)
- physical micro-batch size → PART_OF → ggml graph
- physical micro-batch size → PART_OF → GPU
- logical batching → RELATED_TO → GPU
- -cb (continuous batching) → RELATED_TO → GPU
- -np (number of parallel sequences) → RELATED_TO → parallel decoding
- parallel decoding → RELATED_TO → Key-Value cache
- Speculative Decoding → RELATED_TO → -md /path/to/draft-model.gguf
- Speculative Decoding → RELATED_TO → target model
- Speculative Decoding → RELATED_TO → VRAM
- Speculative Decoding → RELATED_TO → GPU
- --draft-max → RELATED_TO → -md /path/to/draft-model.gguf
- --draft-p-min → RELATED_TO → -md /path/to/draft-model.gguf
- --draft-p-split → RELATED_TO → -md /path/to/draft-model.gguf
- Prompt Lookup Decoding → RELATED_TO → VRAM
- Prompt Lookup Decoding → RELATED_TO → n-grams
- Prompt Lookup Decoding → RELATED_TO → target model
- -lcd (lookup-cache-dynamic) → RELATED_TO → Prompt Lookup Decoding
- -lcs (lookup-cache-static) → RELATED_TO → Prompt Lookup Decoding
- KV Cache Quantization → RELATED_TO → VRAM
- KV Cache Quantization → RELATED_TO → Key-Value cache
- --ctk q8_0 → RELATED_TO → Key-Value cache
- --ctv q8_0 → RELATED_TO → Key-Value cache
- KV Cache Quantization → RELATED_TO → Blackwell-class DGX Spark GB10
- Key-Value cache → RELATED_TO → f16
- Key-Value cache → RELATED_TO → --ctv q8_0
- Key-Value cache → RELATED_TO → q4_0
- TurboQuant → USES → llama.cpp CUDA backend
- TurboQuant → RELATED_TO → Key-Value cache
- turbo4 → RELATED_TO → q4_0
- TurboQuant → RELATED_TO → GPU
- Mean Squared Error (MSE) → RELATED_TO → Quantized Johnson-Lindenstrauss (QJL)
- Sampling Optimization → RELATED_TO → Logit Shaping
- --draft-p-min → RELATED_TO → Top-K samplers
- --draft-p-min → RELATED_TO → Top-P samplers
- --draft-p-min → RELATED_TO → confidence-scaled thresholding
- --draft-p-min → RELATED_TO → probability shaping
- --draft-p-min → RELATED_TO → Top-P
- --temp → RELATED_TO → --draft-p-min
