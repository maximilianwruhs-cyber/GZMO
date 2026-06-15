---
type: entity
title: Physical Batch
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Physical Batch

Type: CONCEPT

## From [[drive-research-llamacpp-optimization-blueprint-micro03|drive-research-llamacpp-optimization-blueprint-micro03]] (2026-06-09)
- Strict physical memory buffer allocated within the ggml graph.
- Represents the actual first dimension of tensor matrices deployed into the GPU's memory pool.
- If restricted, the engine will pipeline the logical batch into sequential physical micro-batches.
