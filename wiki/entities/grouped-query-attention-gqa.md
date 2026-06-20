---
type: entity
title: Grouped Query Attention (GQA)
created: 2026-06-08
updated: 2026-06-10
sources: 6
tags: []
status: draft
gzmo_synthetic: true
---







# Grouped Query Attention (GQA)

Type: CONCEPT

## From [ai-research-part4](/entities/ai-research-part4.md) (2026-06-08)
- Used to reduce KV-Cache requirements.
- Even with GQA, KV-Cache for millions of tokens can reach dozens or hundreds of gigabytes for an 8B-parameter model.

## From [drive-research-agentic-workflows-fastest-best-models](/entities/drive-research-agentic-workflows-fastest-best-models.md) (2026-06-08)
- Drastically reduces the number of KV heads and broadcasts them across query heads.
- Significantly flattens the memory scaling curve compared to MHA.
- Does not solve the fundamental O(N^2) spatial complexity of softmax attention.

## From [ai-research-part6-micro03](/entities/ai-research-part6-micro03.md) (2026-06-09)
- Widely adopted to optimize memory bandwidth during inference.
- A micro-architectural design.

## From [drive-research-32gb-vram-ai-reasoning-models-micro02](/entities/drive-research-32gb-vram-ai-reasoning-models-micro02.md) (2026-06-09)
- Natively implemented by Qwen3-32B.
- Utilizes 64 attention heads for Queries (Q) and only 8 heads for Key-Values (KV).
- Is a vital VRAM-saving mechanism.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro06](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro06.md) (2026-06-09)
- TurboQuant was explicitly designed for standard GQA transformers
- Utilize Euclidean geometric mappings

## From [drive-research-research-process-steps-micro03](/entities/drive-research-research-process-steps-micro03.md) (2026-06-10)
- Query heads are grouped together to share a single key-value head.
- Reduces the size of the KV cache.
