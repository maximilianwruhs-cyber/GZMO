---
type: entity
title: Llama 3.2 1B
created: 2026-06-08
updated: 2026-06-09
sources: 7
tags: []
status: draft
gzmo_synthetic: true
---








# Llama 3.2 1B

Type: BOOK

## From [[drive-research-advanced-inference-acceleration|drive-research-advanced-inference-acceleration]] (2026-06-08)
- A draft model.
- When paired with Llama 3.1 70B, achieves 2x to 3x speedup.

## From [[drive-research-erbandbreite-und-latenzengp-sse|drive-research-erbandbreite-und-latenzengpässe]] (2026-06-08)
- compact draft model
- used with Llama 3.3 70B on NVIDIA H200 yields 3.55x throughput multiplier
- used with Llama 3.1 70B on AMD MI300X yields 2.31x speedup
- used with Qwen 2.5-VL 72B on NVIDIA H200 yields 2.5x speedup

## From [[architectures-and-optimizations-for-speculative-de-micro05|architectures-and-optimizations-for-speculative-de-micro05]] (2026-06-09)
- A lightweight draft model in the Llama 3 ecosystem.
- Provides 2x to 3x speedup against the Llama 3.1 70B target.
- Yields a 1.83x speedup against the Llama 3.1 8B target when drafting 5 tokens per cycle.

## From [[building-a-private-local-ai-development-environmen-micro01|building-a-private-local-ai-development-environmen-micro01]] (2026-06-09)
- Recommended draft model (Junior) for Efficient Workflow with speculative decoding

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro01|the-architecture-of-speculative-decoding-and-infer-part1-micro01]] (2026-06-09)
- A draft model.
- When paired with Llama 3.1 70B, achieves a 2x to 3x speedup.

## From [[the-architecture-of-speculative-decoding-and-infer-part2-micro01|the-architecture-of-speculative-decoding-and-infer-part2-micro01]] (2026-06-09)
- A draft model pairing.
- Paired with Llama 3.3 70B / 3.1 70B.
- Offers lowest latency.

## From [[the-architecture-of-speculative-decoding-and-infer-part2-micro02|the-architecture-of-speculative-decoding-and-infer-part2-micro02]] (2026-06-09)
- Used as a draft model with Llama 3.3 70B.
- Increases throughput to 181.74 tokens per second (3.55x speedup) when paired with Llama 3.3 70B.
- A highly compact draft model.
