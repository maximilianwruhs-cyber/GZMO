---
type: entity
title: Mixture-of-Experts (MoE) architectures
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Mixture-of-Experts (MoE) architectures

Type: ARCHITECTURE

## From [drive-research-linux-gaming-and-ai-build-guide-micro01](/entities/drive-research-linux-gaming-and-ai-build-guide-micro01.md) (2026-06-09)
- The Llama 4 family relies heavily on these architectures.
- Inference is rapid because only a small subset of parameters is active per token.
- The entire parameter set must still be loaded into VRAM.
