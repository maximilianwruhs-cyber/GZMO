---
type: entity
title: logical batching
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# logical batching

Type: CONCEPT

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Interplay between logical batching, physical memory limits, and attention mechanics is highly sensitive.
- Defines the maximum number of tokens processed simultaneously during the pipeline evaluation phase.
- A high value allows the engine to execute massive parallel matrix multiplications during the prompt prefill phase.
