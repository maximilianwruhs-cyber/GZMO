---
type: entity
title: PagedAttention
created: 2026-06-08
updated: 2026-06-10
sources: 7
tags: []
status: draft
gzmo_synthetic: true
---








# PagedAttention

Type: TOOL

## From [[drive-research-erbandbreite-und-latenzengp-sse|drive-research-erbandbreite-und-latenzengpässe]] (2026-06-08)
- An algorithm used by inference servers to manage memory efficiently.
- Partitions KV cache memory into non-contiguous physical blocks.
- Dynamically allocates memory on demand to eliminate fragmentation.

## From [[drive-research-hermes-anthropic-openrouter-cache-investigation|drive-research-hermes-anthropic-openrouter-cache-investigation]] (2026-06-08)
- An optimization technique.
- Used to manage memory footprint efficiently.

## From [[drive-research-32gb-vram-ai-reasoning-models-micro02|drive-research-32gb-vram-ai-reasoning-models-micro02]] (2026-06-09)
- Implemented by vLLM to resolve memory fragmentation.
- Partitions the KV cache into fixed-size, non-contiguous memory blocks.
- Practically eliminates memory fragmentation.

## From [[drive-research-llm-inference-engine-audit-2026-micro02|drive-research-llm-inference-engine-audit-2026-micro02]] (2026-06-09)
- Memory management strategy used by vLLM.
- Exceptionally stable under high-concurrency loads.

## From [[the-architecture-of-speculative-decoding-and-infer-part2-micro02|the-architecture-of-speculative-decoding-and-infer-part2-micro02]] (2026-06-09)
- An algorithm used by vLLM, SGLang, and TensorRT-LLM.
- Partitions memory into non-contiguous physical blocks.
- Dynamically allocates memory on demand to eliminate fragmentation.

## From [[drive-research-llm-inference-engine-audit-2026-micro01|drive-research-llm-inference-engine-audit-2026-micro01]] (2026-06-10)
- Mitigates external fragmentation by dividing KV cache into non-contiguous blocks
- Enables sharing of blocks between sequences

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro03|the-architecture-of-speculative-decoding-and-infer-part1-micro03]] (2026-06-10)
- Partitions the KV cache into fixed-size, non-contiguous memory blocks.
- Inspired by operating system virtual memory paging.
