---
type: entity
title: DenseFormer
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# DenseFormer

Type: SYSTEM

## From [ai-research-part1](/entities/ai-research-part1.md) (2026-06-08)
- Grants each layer access to all previous outputs.
- Combines outputs with fixed, input-independent scalar coefficients.
- Shows no gain over the baseline (1.767) in ablation studies.
- Assigns learned per-layer scalar coefficients fixed after training.
- Uses static weights for cross-layer connectivity.
