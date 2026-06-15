---
type: entity
title: PCIe bus
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# PCIe bus

Type: SYSTEM

## From [[drive-research-cache-optimization-with-ai-chaos-theory|drive-research-cache-optimization-with-ai-chaos-theory]] (2026-06-08)
- Used for Peer-to-Peer (P2P) DMA transfers.
- Enables NCCL to route GPU synchronization directly across it.

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- Page-locked memory allows the GPU's onboard DMA engines to copy weights directly across the PCIe bus.
- This bypasses the host CPU entirely and achieves maximum theoretical hardware bandwidth.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro05|optimizing-nvidia-blackwell-sm120-part2-micro05]] (2026-06-10)
- Can introduce latency during tensor splitting in multi-GPU arrays
