---
type: entity
title: MiniCPM-SALA
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# MiniCPM-SALA

Type: SYSTEM

## From [ai-research-part4](/entities/ai-research-part4.md) (2026-06-08)
- A 9B-parameter hybrid architecture.
- Integrates InfLLM-V2 and Lightning Attention.
- Achieves up to 3.5x inference speed of full-attention models at 256K tokens.
- Supports context lengths up to 1M tokens.
- Exhibits surprising length extrapolation capabilities.
- Achieves a score of 23.86 at the 128K level in the NoLiMa benchmark.
- Demonstrates a significant performance advantage over Qwen3-8B across all tested configurations in inference speed.

## From [ai-research-part8-micro03](/entities/ai-research-part8-micro03.md) (2026-06-09)
- Approaches the long-context bottleneck through a 9B-parameter hybrid architecture.
- Blends 25% Sparse Attention with 75% Linear Attention.
- Can reliably process up to 2 million tokens.
- Logged a 3.5x inference speedup over dense models like Qwen3-8B at 256K context.

## From [ai-research-part8-micro05](/entities/ai-research-part8-micro05.md) (2026-06-09)
- A large-scale hybrid model.
- Integrates sparse attention with linear attention.
- Processes contexts exceeding one million tokens.
- Demonstrates architectural shifts to bypass the 'Memory Wall'.
- The MiniCPM-SALA model integrates sparse and linear attention.
- Successfully processes contexts exceeding one million tokens.

## From [ai-research-part8-micro07](/entities/ai-research-part8-micro07.md) (2026-06-09)
- It is a leading AI framework for Foundation Architectures.
- It is associated with SALA.
- It is a framework for breaking the context wall.
- It uses Hybrid Sparse-Linear topologies.
