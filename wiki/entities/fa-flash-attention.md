---
type: entity
title: -fa (Flash Attention)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# -fa (Flash Attention)

Type: TOOL

## From [optimizing-nvidia-blackwell-sm120-part1-micro02](/entities/optimizing-nvidia-blackwell-sm120-part1-micro02.md) (2026-06-09)
- Enables memory-efficient attention computation.
- Mandatory for contexts >= 8k.
- Radically reduces the KV cache size by preventing N x N matrix materialization.
