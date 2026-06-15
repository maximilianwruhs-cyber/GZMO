---
type: entity
title: -cb (continuous batching)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# -cb (continuous batching)

Type: TOOL

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Orchestrates multi-stream processing.
- Mitigates idle compute cycles by actively searching the server queue for incoming client requests, grouping them together, and injecting them into the active logical batch layers dynamically.
- Ensures that the GPU pipeline remains fully saturated across all layer computations.
