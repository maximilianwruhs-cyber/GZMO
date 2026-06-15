---
type: entity
title: Paged KV Cache
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Paged KV Cache

Type: CONCEPT

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- Mirrors virtual memory paging in operating systems.
- Allocates physical blocks on-demand from a shared pool.
- Uses a dynamic block table to map memory.
- Stores computed attention states of past tokens to avoid redundant calculations.
- Unified KV cache allocates static, contiguous blocks of VRAM.
- Paged KV Cache scheduler divides the cache into small, fixed-size blocks.
