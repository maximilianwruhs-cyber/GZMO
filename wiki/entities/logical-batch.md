---
type: entity
title: Logical Batch
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Logical Batch

Type: CONCEPT

## From [drive-research-llamacpp-optimization-blueprint-micro03](/entities/drive-research-llamacpp-optimization-blueprint-micro03.md) (2026-06-09)
- Defines the maximum number of tokens processed simultaneously during pipeline evaluation.
- A high value allows the engine to execute massive parallel matrix multiplications.
- Pipelined into physical micro-batches if restricted by physical memory.
