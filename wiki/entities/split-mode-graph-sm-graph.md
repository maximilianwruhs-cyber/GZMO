---
type: entity
title: Split Mode Graph (-sm graph)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Split Mode Graph (-sm graph)

Type: CONCEPT

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- A mode used by llama.cpp for multi-GPU execution.
- Implements tensor parallelism at the GGML graph level.
- Distributes compute graph nodes across GPUs.
