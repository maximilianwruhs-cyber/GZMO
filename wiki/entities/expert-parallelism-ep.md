---
type: entity
title: Expert Parallelism (EP)
created: 2026-06-08
updated: 2026-06-08
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Expert Parallelism (EP)

Type: CONCEPT

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- Distributes entire expert networks to distinct GPUs.
- Reduces inter-GPU communication to simple routing tokens.
- Enables efficient scaling across node boundaries.
- It is utilized by vLLM for low-latency production serving.
- It distributes entire expert networks to distinct physical GPUs.
- It partitions the KV cache, saving significant HBM.

## From [[drive-research-frankenmoe-blueprint-analysis|drive-research-frankenmoe-blueprint-analysis]] (2026-06-08)
- Distributes entire expert networks to distinct physical GPUs.
- Used in low-latency production serving via vLLM.
- Saves significant High-Bandwidth Memory (HBM) compared to Tensor Parallelism.

## From [[drive-research-optimizing-qwen36-on-blackwell-gpus|drive-research-optimizing-qwen36-on-blackwell-gpus]] (2026-06-08)
- Shards MoE expert weights across GPUs while replicating standard attention weights.
- Token routing requires an all-to-all communication dispatch and gather phase.
- Can quickly saturate the PCIe interface in desktop configurations.
