---
type: entity
title: DenseNet
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---



# DenseNet

Type: SYSTEM

## From [[ai-research-part1|ai-research-part1]] (2026-06-08)
- A residual update mechanism.
- Uses a static weight.
- Can access [h1, ..., hl-1] as sources.
- Concatenates all preceding feature maps.
- Uses static weights for cross-layer connectivity.

## From [[ai-research-part6-micro03|ai-research-part6-micro03]] (2026-06-09)
- A foundational residual architecture.
- Demonstrated that explicit shortcut or gating pathways substantially ease optimization and improve depth scalability.
