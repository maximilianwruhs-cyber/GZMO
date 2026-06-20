---
type: entity
title: Triton kernels
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Triton kernels

Type: TOOL

## From [drive-research-speicherbandbreiten-engpass-memory-wall](/entities/drive-research-speicherbandbreiten-engpass-memory-wall.md) (2026-06-08)
- Provide deep integration in vLLM for high-throughput serving.
- Pre-rotate the Query (Q) vector with the inverse of the PolarQuant rotation matrix.
- Compute attention scores by directly gathering centroid values via table lookups.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro06](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro06.md) (2026-06-09)
- Specialized kernels used by vLLM for deep integration
- Designed for vLLM to bypass the hybrid decode penalty
- Operate uniquely to bypass the hybrid decode penalty
- Pre-rotate the Query (Q) vector with the inverse of the PolarQuant rotation matrix via a single matrix multiplication
- Compute attention scores in parallel across all sequence positions by directly gathering centroid values via table lookups
- Utilize the 8-bit unsigned integer (uint8) packed indices stored in the cache
- Never materialize the FP16 keys
- Move roughly 4x less data from GPU memory
- Achieve massive speedups under memory pressure
