---
type: entity
title: Mixture-of-Depths Attention (MoDA)
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Mixture-of-Depths Attention (MoDA)

Type: CONCEPT

## From [[ai-research-part5|ai-research-part5]] (2026-06-08)
- Acronym for Mixture-of-Depths Attention.
- A unified attention mechanism.
- Improves average perplexity and downstream performance.
- A hardware-aware design that reorganizes depth KV layout and fuses computation.
- Aims to reduce non-contiguous memory access and improve effective compute utilization.
- Has flash-compatible, chunk-aware, and group-aware variants.
- A unified depth-aware attention mechanism for LLM.
- Improves depth-wise information aggregating.
- Mitigates depth-efficiency gaps.

## From [[ai-research-part8-micro07|ai-research-part8-micro07]] (2026-06-09)
- It solves PreNorm Dilution by augmenting fixed depth accumulation with dynamic attention.
- It uses softmax-weighted attention across previous layers.

## From [[ai-research-part8-micro08|ai-research-part8-micro08]] (2026-06-09)
- Part of 'Lösung der PreNorm-Dilution'.
- Allows attention heads to jointly attend to current sequence key-value pairs as well as depth key-value pairs from preceding layers.
- Authored by Zhu et al., ByteDance/HUST.
