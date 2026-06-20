---
type: entity
title: Tensor Parallelism (TP)
created: 2026-06-08
updated: 2026-06-08
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Tensor Parallelism (TP)

Type: CONCEPT

## From [architectural-blueprints-for-sovereign-frankenmoe-part1](/entities/architectural-blueprints-for-sovereign-frankenmoe-part1.md) (2026-06-08)
- Shards weight matrices across multiple GPUs.
- Requires high-bandwidth interconnects (e.g., NVLink).
- Contrasted with Expert Parallelism.
- It shards weight matrices across physical silicon.
- It saturates inter-GPU linkages.
- Under TP=8, the KV cache must be fully duplicated across all 8 GPUs.

## From [drive-research-frankenmoe-blueprint-analysis](/entities/drive-research-frankenmoe-blueprint-analysis.md) (2026-06-08)
- Shards weight matrices across physical silicon.
- Saturates inter-GPU linkages.
- Requires KV cache to be fully duplicated across GPUs.

## From [drive-research-optimizing-qwen36-on-blackwell-gpus](/entities/drive-research-optimizing-qwen36-on-blackwell-gpus.md) (2026-06-08)
- Splits individual matrix multiplication operations across both GPUs.
- Requires frequent, high-latency AllReduce synchronization steps.
- Running a tensor-parallel size of 1 (TP=1) is recommended for PCIe bottlenecking avoidance.
