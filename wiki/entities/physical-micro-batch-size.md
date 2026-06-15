---
type: entity
title: physical micro-batch size
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# physical micro-batch size

Type: CONCEPT

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Defines the strict physical memory buffer allocated within the ggml graph.
- The ubatch represents the actual first dimension of the tensor matrices deployed into the GPU's memory pool.
