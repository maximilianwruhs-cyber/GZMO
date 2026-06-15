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
- [[draft-p-min|--draft-p-min]] (CONCEPT)
- [[top-k-samplers|Top-K samplers]] (CONCEPT)
- [[speculative-decoding|Speculative Decoding]] (CONCEPT)
- [[n-grams|n-grams]] (CONCEPT)
- [[confidence-scaled-thresholding|confidence-scaled thresholding]] (CONCEPT)
- [[turbo4|turbo4]] (TOOL)
- [[top-p-samplers|Top-P samplers]] (TOOL)
- [[sampler-seq|--sampler-seq]] (TOOL)
- [[llama-cpp-cuda-backend|llama.cpp CUDA backend]] (SYSTEM)
- [[logical-batching|logical batching]] (CONCEPT)
- [[kv-cache-quantization|KV Cache Quantization]] (CONCEPT)
- [[optimizing-nvidia-blackwell-sm120-part1|optimizing-nvidia-blackwell-sm120-part1]] (PROJECT)
- [[target-model|target model]] (CONCEPT)
- [[sampling-optimization|Sampling Optimization]] (CONCEPT)
- [[key-value-cache|Key-Value cache]] (CONCEPT)
- [[draft-max|--draft-max]] (TOOL)
- [[gpu|GPU]] (SYSTEM)
- [[attention-mechanics|attention mechanics]] (CONCEPT)
- [[quantized-johnson-lindenstrauss-qjl|Quantized Johnson-Lindenstrauss (QJL)]] (CONCEPT)
- [[logical-batch-size|logical batch size]] (CONCEPT)
- [[physical-micro-batch-size|physical micro-batch size]] (CONCEPT)
- [[probability-shaping|probability shaping]] (CONCEPT)
- [[ctv-q8-0|--ctv q8_0]] (TOOL)
- [[retrieval-augmented-generation-rag|Retrieval-Augmented Generation (RAG)]] (CONCEPT)
- [[document-summarization|document summarization]] (CONCEPT)
- [[coding-tasks|coding tasks]] (CONCEPT)
- [[temperature-scaling|temperature scaling]] (CONCEPT)
- [[f16|f16]] (CONCEPT)
- [[mean-squared-error-mse|Mean Squared Error (MSE)]] (CONCEPT)
- [[ggml-graph|ggml graph]] (SYSTEM)
- [[ub-physical-batch|-ub (Physical Batch)]] (TOOL)
- [[cb-continuous-batching|-cb (continuous batching)]] (TOOL)
- [[turboquant|TurboQuant]] (CONCEPT)
- [[q4-0|q4_0]] (CONCEPT)
- [[prompt-lookup-decoding|Prompt Lookup Decoding]] (CONCEPT)
- [[blackwell-class-dgx-spark-gb10|Blackwell-class DGX Spark GB10]] (SYSTEM)
- [[lloyd-max-scalar-quantization|Lloyd-Max scalar quantization]] (CONCEPT)
- [[top-p|Top-P]] (CONCEPT)
- [[parallel-decoding|parallel decoding]] (CONCEPT)
- [[c-context|-c (Context)]] (TOOL)
- [[physical-memory-limits|physical memory limits]] (CONCEPT)
- [[logit-shaping|Logit Shaping]] (CONCEPT)
- [[cuda-cores|CUDA cores]] (SYSTEM)
- [[fa-flash-attention|-fa (Flash Attention)]] (TOOL)
- [[md-path-to-draft-model-gguf|-md /path/to/draft-model.gguf]] (TOOL)
- [[draft-p-split|--draft-p-split]] (TOOL)
- [[temp|--temp]] (TOOL)
- [[turbo3|turbo3]] (TOOL)
- [[lcs-lookup-cache-static|-lcs (lookup-cache-static)]] (TOOL)
- [[ctk-q8-0|--ctk q8_0]] (TOOL)
- [[vram|VRAM]] (CONCEPT)
- [[lcd-lookup-cache-dynamic|-lcd (lookup-cache-dynamic)]] (TOOL)
- [[batching-layer|batching layer]] (CONCEPT)
- [[np-number-of-parallel-sequences|-np (number of parallel sequences)]] (TOOL)
- [[rtx-3090|RTX 3090]] (SYSTEM)
- [[b-logical-batch|-b (Logical Batch)]] (TOOL)

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
