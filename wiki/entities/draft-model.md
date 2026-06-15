---
type: entity
title: Draft Model
created: 2026-06-08
updated: 2026-06-09
sources: 6
tags: []
status: draft
gzmo_synthetic: true
---






# Draft Model

Type: CONCEPT

## From [[drive-research-advanced-inference-acceleration|drive-research-advanced-inference-acceleration]] (2026-06-08)
- A small, computationally light, and extremely fast model.
- Generates a short sequence of candidate tokens autoregressively.
- Used in speculative decoding.

## From [[drive-research-enhancing-local-ai-hypervisor-architecture|drive-research-enhancing-local-ai-hypervisor-architecture]] (2026-06-08)
- Is a fast, parameter-light Small Language Model (SLM).
- Rapidly generates a sequence of candidate tokens.
- Can be an EAGLE-based model.

## From [[drive-research-speicherbandbreiten-engpass-memory-wall|drive-research-speicherbandbreiten-engpass-memory-wall]] (2026-06-08)
- A smaller model used in Speculative Decoding to rapidly generate candidate tokens.
- Can be loaded on VRAM-constrained GPUs thanks to TurboQuant.
- Can be a Weight-Only-Quantized (WOQ) model in ML-SpecQD.

## From [[building-a-private-local-ai-development-environmen-micro06|building-a-private-local-ai-development-environmen-micro06]] (2026-06-09)
- An ultra-fast, microscopic predicting model.
- Guesses the next 5 to 10 words of code.
- Also referred to as the 'Junior Dev'.

## From [[drive-research-llamacpp-optimization-blueprint-micro03|drive-research-llamacpp-optimization-blueprint-micro03]] (2026-06-09)
- A smaller model used in speculative decoding.
- Has the identical architecture family as the target model.
- Evaluates quickly, generating a sequence of K tokens.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro01|the-architecture-of-speculative-decoding-and-infer-part1-micro01]] (2026-06-09)
- A small, computationally light, and extremely fast model used in speculative decoding.
- Generates a short sequence of candidate tokens autoregressively.
- Its predictions are verified by the target model.
