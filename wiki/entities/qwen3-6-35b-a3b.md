---
type: entity
title: Qwen3.6-35B-A3B
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Qwen3.6-35B-A3B

Type: MODEL

## From [[drive-research-optimizing-qwen36-on-blackwell-gpus|drive-research-optimizing-qwen36-on-blackwell-gpus]] (2026-06-08)
- A large language model with 35.1 billion total parameters.
- Utilizes a highly sparse Mixture of Experts (MoE) combined with a hybrid attention mechanism.
- Activates approximately 3 billion parameters per token.

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro02|drive-research-cuda-graph-capture-failure-workarounds-micro02]] (2026-06-09)
- A high-capacity Mixture of Experts (MoE) architecture.
- Can be deployed on hardware configurations with constrained physical memory.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro05|optimizing-nvidia-blackwell-sm120-part3-micro05]] (2026-06-09)
- An example of a high-capacity Mixture of Experts architecture.
- Deployed on hardware with constrained physical memory.
- Exposes critical defects in context checkpoint allocation.
