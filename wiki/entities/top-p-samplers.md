---
type: entity
title: Top-P samplers
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Top-P samplers

Type: TOOL

## From [optimizing-nvidia-blackwell-sm120-part1-micro02](/entities/optimizing-nvidia-blackwell-sm120-part1-micro02.md) (2026-06-09)
- Dictates the exact execution order of samplers in llama.cpp.
- Custom tuning of this sequence allows architects to fine-tune the exact statistical pipeline.
- Legacy samplers that calculate a cumulative probability mass and cull the tail.
- Must be disabled when using Min-P.
