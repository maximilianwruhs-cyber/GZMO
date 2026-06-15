---
type: entity
title: MoE I/O Prefetching (MoE-SpeQ)
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# MoE I/O Prefetching (MoE-SpeQ)

Type: CONCEPT

## From [[drive-research-advanced-inference-acceleration|drive-research-advanced-inference-acceleration]] (2026-06-08)
- A framework for Sparse Mixture of Experts (MoE) models.
- A tiny draft model predicts the expert sequence.
- Enables Expert Lookahead Buffer (ELB) to preload expert weights asynchronously.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro01|the-architecture-of-speculative-decoding-and-infer-part1-micro01]] (2026-06-09)
- A technique for Sparse Mixture of Experts (MoE) models.
- A tiny draft model predicts the expert sequence needed for future tokens.
- Allows an Expert Lookahead Buffer (ELB) to preload expert weights asynchronously.
- Achieves up to 2.34x speedups by masking I/O latency.
