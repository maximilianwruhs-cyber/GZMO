---
type: entity
title: DeepNorm
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# DeepNorm

Type: SYSTEM

## From [ai-research-part1](/entities/ai-research-part1.md) (2026-06-08)
- A residual update mechanism.
- Uses a fixed weight.
- Accesses only hl−1 as a source.

## From [ai-research-part6-micro02](/entities/ai-research-part6-micro02.md) (2026-06-09)
- A Post-Norm variant with depth-dependent residual scaling and initialization.
- Demonstrates superior stability by successfully converging at a learning rate of 1x10^-3.
- Eventually succumbs to instability and diverges at 2x10^-3.
