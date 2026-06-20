---
type: entity
title: VRAM Bandwidth Saturation
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# VRAM Bandwidth Saturation

Type: CONCEPT

## From [drive-research-ok-so-designing-a-guide-around-llamabench-would-b](/entities/drive-research-ok-so-designing-a-guide-around-llamabench-would-b.md) (2026-06-08)
- Token Generation is entirely memory-bandwidth bound.
- Every token generated forces the GPU to read the entire active model weight matrix out of VRAM.
- Achieved memory read speed can be calculated using llama-bench throughput.
