---
type: entity
title: Gemma 4 26B
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Gemma 4 26B

Type: MODEL

## From [drive-research-agentic-workflows-fastest-best-models](/entities/drive-research-agentic-workflows-fastest-best-models.md) (2026-06-08)
- Introduces a Sparse Mixture of Experts (MoE) architecture.
- Physically stores 26 billion parameters but routes tokens to only 2 active experts.
- Drastically reduces active parameter count per forward pass.
- Excels in rapid general reasoning.
- Utilizes a 1024-token sliding window attention to support a 256,000-token context.
- Extreme sparsity limits its structural rigidity.
- Struggles to maintain global attention state for complex JSON schemas.
