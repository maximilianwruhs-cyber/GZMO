---
type: entity
title: GGML_CUDA_NCCL=ON
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# GGML_CUDA_NCCL=ON

Type: SYSTEM

## From [[drive-research-cache-optimization-with-ai-chaos-theory|drive-research-cache-optimization-with-ai-chaos-theory]] (2026-06-08)
- Allows only one active CUDA context to exist on the device at any time when in EXCLUSIVE_PROCESS mode.
- Context initialization is lazy.
- Runtime compilation flag to achieve linear scaling in Graph Split mode.
- Configures NVIDIA Collective Communications Library (NCCL).
- Returns hard `cudaErrorDeviceUnavailable` error to blockers when EXCLUSIVE_PROCESS mode is active.
- Allows only one active CUDA context to exist on the device at any time.
- Environment variable to enable Peer-to-Peer (P2P) DMA transfers over the PCIe bus.
- Allows NCCL to bypass system memory and route GPU synchronization directly across the bus.
