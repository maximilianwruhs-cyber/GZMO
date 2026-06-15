---
type: entity
title: Pipeline (Layer) Parallelism
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Pipeline (Layer) Parallelism

Type: CONCEPT

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- Sequentially distributes transformer layers across available GPU memory.
- Introduces execution pipeline gaps and GPU idling.
