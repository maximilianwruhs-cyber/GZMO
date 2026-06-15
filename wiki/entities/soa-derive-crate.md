---
type: entity
title: soa_derive Crate
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# soa_derive Crate

Type: TOOL

## From [[drive-research-cache-optimization-with-ai-chaos-theory|drive-research-cache-optimization-with-ai-chaos-theory]] (2026-06-08)
- Automatically generates parallel vectors with separate heap allocations per struct field.
- Employs Generic Associated Types (GATs) to generate custom Slice/Ref helper types.
- Demonstrated 6x speedup over AoS layouts in vector arithmetic and dot product tasks.

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- Automatically generates parallel vectors with separate allocations per field.
- Employs Generic Associated Types (GATs) to generate custom Slice/Ref helper types.
- Demonstrated 6x speedup over AoS layouts in vector arithmetic tasks.
