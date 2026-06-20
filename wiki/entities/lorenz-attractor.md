---
type: entity
title: Lorenz Attractor
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Lorenz Attractor

Type: CONCEPT

## From [gzmo-chaos-engine-architecture-audit-and-behaviora](/entities/gzmo-chaos-engine-architecture-audit-and-behaviora.md) (2026-06-08)
- Used in the GZMO Chaos Engine's signal chain
- RK4 step (dt=0.005), σ=10, ρ=28, β=8/3
- Influences Temperature, Valence, and MaxTokens

## From [drive-research-tinyfolder-gzmo-architecture-analysis-product](/entities/drive-research-tinyfolder-gzmo-architecture-analysis-product.md) (2026-06-08)
- Used to modulate temperature, tokens, and valence.
- RK4 integration for deterministic chaos.
