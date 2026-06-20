---
type: entity
title: Graph Reuse (-gr)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Graph Reuse (-gr)

Type: CONCEPT

## From [drive-research-rust-ecs-cache-optimization-research](/entities/drive-research-rust-ecs-cache-optimization-research.md) (2026-06-08)
- Enables immediate recycling of computed GGML graph allocations across steps.
- Provides a slight reduction in latency per token.
- Compatible across CPU and GPU backends.
