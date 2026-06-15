---
type: entity
title: Token Generation
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Token Generation

Type: CONCEPT

## From [[drive-research-llamabench|drive-research-llamabench]] (2026-06-08)
- Also called the decode phase
- Measures the speed of generating subsequent tokens sequentially
- Heavily bound by memory bandwidth

## From [[drive-research-llama-bench-performance-benchmarking-tool-micro01|drive-research-llama-bench-performance-benchmarking-tool-micro01]] (2026-06-09)
- Commonly called the decode phase.
- Processes subsequent tokens sequentially in an autoregressive loop.
- Memory-bandwidth-bound.
- Throughput measured in tokens per second (t/s).
