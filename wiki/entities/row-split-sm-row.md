---
type: entity
title: Row Split (-sm row)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Row Split (-sm row)

Type: CONCEPT

## From [drive-research-cache-optimization-with-ai-chaos-theory](/entities/drive-research-cache-optimization-with-ai-chaos-theory.md) (2026-06-08)
- Splits weight matrices across devices; every GPU contains a portion of every layer.
- Achieves balanced GPU utilization through parallelized matrix multiplication on GPU rows.
- Evenly distributes massive KV cache allocations to prevent individual VRAM exhaustion.
- Requires high-speed interconnects (NVLink/PCIe Gen 5) to prevent bus bottlenecks.
