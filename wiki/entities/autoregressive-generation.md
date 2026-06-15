---
type: entity
title: Autoregressive Generation
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# Autoregressive Generation

Type: CONCEPT

## From [[drive-research-advanced-inference-acceleration|drive-research-advanced-inference-acceleration]] (2026-06-08)
- The standard generation method where each token is produced sequentially.
- Each token prediction requires a full forward pass.
- Leads to low arithmetic intensity and is memory-bandwidth bound.

## From [[architectures-and-optimizations-for-speculative-de-micro04|architectures-and-optimizations-for-speculative-de-micro04]] (2026-06-09)
- Each token is produced sequentially.
- Model must complete a full forward pass for each token.
- Forces AI accelerators into a regime of drastically low arithmetic intensity.
- Operation is heavily memory-bandwidth bound rather than compute-bound.

## From [[architectures-and-optimizations-for-speculative-de-micro06|architectures-and-optimizations-for-speculative-de-micro06]] (2026-06-09)
- Standard generation process that introduces latency when loading expert weights.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro01|the-architecture-of-speculative-decoding-and-infer-part1-micro01]] (2026-06-09)
- The standard generation method where each token is generated sequentially.
- Each token prediction requires a full forward pass.
- This sequential dependency limits modern AI accelerators to low arithmetic intensity.
- The process is memory-bandwidth bound, not compute-bound.
