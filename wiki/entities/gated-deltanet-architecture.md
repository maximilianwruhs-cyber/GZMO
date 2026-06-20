---
type: entity
title: Gated DeltaNet architecture
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Gated DeltaNet architecture

Type: CONCEPT

## From [drive-research-agentic-workflows-fastest-best-models](/entities/drive-research-agentic-workflows-fastest-best-models.md) (2026-06-08)
- Implemented in Qwen3.6-27B.
- A departure from pure self-attention.
- Consists of repeating hybrid blocks of DeltaNet layers and Gated Attention layers.
- An advanced linear attention variant.
- Processes data sequentially and compresses historical context into a fixed-size recurrent state.
- Replaces 75% of standard self-attention layers, eviscerating KV cache requirements.
- A component of the Gated DeltaNet architecture.
