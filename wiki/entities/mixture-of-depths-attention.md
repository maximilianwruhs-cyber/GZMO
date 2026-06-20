---
type: entity
title: Mixture-of-Depths Attention
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---




# Mixture-of-Depths Attention

Type: CONCEPT

## From [ai-research-part5](/entities/ai-research-part5.md) (2026-06-08)
- A mechanism to address LLM information dilution.
- Allows attention heads to attend to sequence KV pairs and depth KV pairs from preceding layers.
- Fuses sequence and depth attention into a single operator.

## From [ai-research-part8-micro02](/entities/ai-research-part8-micro02.md) (2026-06-09)
- An alternative resolution to the depth dilution crisis.
- Upgrades the sequence-mixing attention matrix to jointly attend over sequence and depth dimensions.
- Introduces a group reordering strategy and chunk-aware memory layouts.

## From [ai-research-part8-micro04](/entities/ai-research-part8-micro04.md) (2026-06-09)
- An innovation at the base architectural layer for AI.
- Fundamentally re-engineered how information propagates across massive sequences.
