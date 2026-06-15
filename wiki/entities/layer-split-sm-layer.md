---
type: entity
title: Layer Split (-sm layer)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Layer Split (-sm layer)

Type: CONCEPT

## From [[drive-research-cache-optimization-with-ai-chaos-theory|drive-research-cache-optimization-with-ai-chaos-theory]] (2026-06-08)
- Sequentially distributes transformer layers across available GPU memory.
- Results in low GPU utilization due to execution pipeline gaps.
- KV cache spreads across devices alongside their assigned layers.
- Universally supported across CUDA, Vulkan, and Metal backends.
