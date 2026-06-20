---
type: entity
title: Llama 3.3 70B
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Llama 3.3 70B

Type: SYSTEM

## From [the-architecture-of-speculative-decoding-and-infer-part2-micro02](/entities/the-architecture-of-speculative-decoding-and-infer-part2-micro02.md) (2026-06-09)
- A massive target model.
- Standard autoregressive inference without speculation achieves approximately 51.14 tokens per second on an NVIDIA H200 GPU.
- When used with a 1-billion parameter draft model, yields 181.74 tokens per second (3.55x speedup).
- A target model.
- When used with Llama 3.2 1B on an AMD MI300X, yields 2.31x speedup.

## From [the-architecture-of-speculative-decoding-and-infer-part2-micro03](/entities/the-architecture-of-speculative-decoding-and-infer-part2-micro03.md) (2026-06-09)
- Can be a target model in speculative decoding.
