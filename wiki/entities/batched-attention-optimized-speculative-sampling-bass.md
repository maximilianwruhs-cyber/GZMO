---
type: entity
title: Batched Attention-optimized Speculative Sampling (BASS)
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Batched Attention-optimized Speculative Sampling (BASS)

Type: TOOL

## From [[drive-research-erbandbreite-und-latenzengp-sse|drive-research-erbandbreite-und-latenzengpässe]] (2026-06-08)
- implements customized CUDA kernels
- engineered to handle ragged tensors directly during attention calculation
- establishes state-of-the-art multi-sequence generation latency while preserving peak GPU utilization

## From [[the-architecture-of-speculative-decoding-and-infer-part2-micro02|the-architecture-of-speculative-decoding-and-infer-part2-micro02]] (2026-06-09)
- Implements customized CUDA kernels.
- Engineered to handle ragged tensors directly during attention calculation.
- Establishes state-of-the-art multi-sequence generation latency.
