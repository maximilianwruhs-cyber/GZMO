---
type: entity
title: -ub (Physical Batch)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# -ub (Physical Batch)

Type: TOOL

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Physical micro-batch memory size.
- Must strictly be <= -b.
- Adjust this downward if you hit VRAM memory spikes during prompt processing.
- Defines the strict physical memory buffer allocated within the ggml graph.
