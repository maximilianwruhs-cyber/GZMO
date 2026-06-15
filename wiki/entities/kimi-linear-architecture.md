---
type: entity
title: Kimi Linear architecture
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Kimi Linear architecture

Type: SYSTEM

## From [[ai-research-part1|ai-research-part1]] (2026-06-08)
- Our architecture is identical to Kimi Linear [69].
- A Mixture-of-Experts (MoE) Transformer following the Moonlight [28] / DeepSeek-V3 [9] design.
- Interleaves Kimi Delta Attention (KDA) and Multi-Head Latent Attention (MLA) layers in a 3:1 ratio.
- A model architecture with 48B total / 3B activated parameters.
- AttnRes was integrated into this architecture.
- Pre-trained on 1.4T tokens.

## From [[ai-research-part8-micro02|ai-research-part8-micro02]] (2026-06-09)
- Integrated AttnRes into its 48B-parameter design.
- Used Mixture-of-Experts routing for 3B activated parameters.

## From [[ai-research-part8-micro03|ai-research-part8-micro03]] (2026-06-09)
- Developed to manage 48B total parameters with 3B active parameters via a Mixture-of-Experts routing.
- Achieved an extraordinary 75% reduction in Key-Value (KV) cache footprint compared to standard full-attention models.
- Sustained a decoding throughput 6x higher than standard MLA baselines during 1M-token context inference.
