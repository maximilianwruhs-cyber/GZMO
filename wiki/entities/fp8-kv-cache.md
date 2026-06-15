---
type: entity
title: FP8 KV cache
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# FP8 KV cache

Type: CONCEPT

## From [[drive-research-agentic-workflows-fastest-best-models|drive-research-agentic-workflows-fastest-best-models]] (2026-06-08)
- Halves the memory requirement of in-flight requests.
- Degradation in generation quality is practically negligible.
- Maintains over 99% of unquantized accuracy baseline.
- Uses e4m3 configuration for attention matrix multiplications.
- Definitive, mathematically sound mechanism for extending context boundary.
- Enforced within the inference engine.
- Safeguards remaining VRAM budget.
- Doubles permissible context length.
