---
type: entity
title: Prompt Processing
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Prompt Processing

Type: CONCEPT

## From [drive-research-llamabench](/entities/drive-research-llamabench.md) (2026-06-08)
- Also called the prefill phase
- Measures how fast hardware computes the initial KV cache
- Heavily bound by compute performance (FLOPs) and matrix multiplication efficiency

## From [drive-research-llama-bench-performance-benchmarking-tool-micro01](/entities/drive-research-llama-bench-performance-benchmarking-tool-micro01.md) (2026-06-09)
- Often referred to as the prefill phase.
- Processes the initial input sequence to compute the starting Key-Value (KV) cache.
- Dominated by dense General Matrix-Matrix Multiplications (GEMM).
- Heavily compute-bound, scaling with hardware's raw floating-point execution capacity.
