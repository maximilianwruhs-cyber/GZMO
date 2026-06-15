---
type: entity
title: Chunk-aware depth KV layout
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Chunk-aware depth KV layout

Type: CONCEPT

## From [[ai-research-part5|ai-research-part5]] (2026-06-08)
- Queries are divided into chunks, and each chunk accesses a corresponding depth-KV span.
- Reduces unnecessary HBM traffic and improves depth utilization.
