---
type: entity
title: NVIDIA Collective Communications Library (NCCL)
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# NVIDIA Collective Communications Library (NCCL)

Type: SYSTEM

## From [[drive-research-cache-optimization-with-ai-chaos-theory|drive-research-cache-optimization-with-ai-chaos-theory]] (2026-06-08)
- Configured by GGML_CUDA_NCCL=ON.
- Bypasses system memory and routes GPU synchronization directly across the bus when P2P is enabled.

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- Relied upon by llama.cpp for linear scaling in Graph Split mode.
- Auto-detects physical topology of the PCIe bus.
- Establishes direct Peer-to-Peer (P2P) DMA transfers between GPUs.
