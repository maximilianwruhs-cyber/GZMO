---
type: entity
title: Asymmetric Quantization
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Asymmetric Quantization

Type: CONCEPT

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro06](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro06.md) (2026-06-09)
- Critical optimization deployed in vLLM setups
- Empirical audits of TurboQuant revealed that Key (K) and Value (V) matrices exhibit differing sensitivities to quantization noise
- Optimal vLLM configurations allocate mixed precisions: 3-bit for Keys and 4-bit for Values
