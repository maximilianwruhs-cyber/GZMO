---
type: entity
title: INT8
created: 2026-06-09
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# INT8

Type: CONCEPT

## From [drive-research-32gb-vram-ai-reasoning-models-micro01](/entities/drive-research-32gb-vram-ai-reasoning-models-micro01.md) (2026-06-09)
- 8-bit quantization
- Each parameter requires 1 byte of memory
- Can be used for KV cache quantization

## From [drive-research-32gb-vram-ai-reasoning-models-micro03](/entities/drive-research-32gb-vram-ai-reasoning-models-micro03.md) (2026-06-09)
- Quantization level for KV cache.
- Structural necessity for pushing context lengths toward 128K or 256K.

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- A Tensor Core format supported by SM120 and SM121.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro04](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro04.md) (2026-06-09)
- Can be used for quantizing the KV cache.
