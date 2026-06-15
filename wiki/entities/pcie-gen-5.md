---
type: entity
title: PCIe Gen 5
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# PCIe Gen 5

Type: CONCEPT

## From [[drive-research-optimizing-qwen36-on-blackwell-gpus|drive-research-optimizing-qwen36-on-blackwell-gpus]] (2026-06-08)
- Desktop Blackwell GPUs must communicate strictly over PCIe Gen 5 x16 slots.
- Tensor Parallelism over PCIe links severely limits performance.
- Pipeline Parallelism is highly stable and avoids inter-GPU synchronization bottlenecks typical of PCIe setups.
