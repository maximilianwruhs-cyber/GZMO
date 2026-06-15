---
type: entity
title: AMD RDNA hardware
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# AMD RDNA hardware

Type: SYSTEM

## From [[drive-research-llamacpp-gpu-memory-reporting-bug|drive-research-llamacpp-gpu-memory-reporting-bug]] (2026-06-08)
- Half-precision reduction operations produced incorrect results.
- Caused causal mask block-skipping logic to skip valid attention positions.
