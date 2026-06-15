---
type: entity
title: page-locked memory
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# page-locked memory

Type: CONCEPT

## From [[drive-research-cuda-memory-locking-limits-configuration|drive-research-cuda-memory-locking-limits-configuration]] (2026-06-08)
- Also known as pinned memory.
- Used within the NVIDIA CUDA architecture to accelerate host-to-device transfers.
- Standard memory allocated via malloc or anonymous mmap is pageable and requires the CUDA driver to copy data into an internal, page-locked staging buffer.
- cudaHostRegister registers an existing host virtual address range as page-locked memory.
- The NVIDIA driver locks these pages in physical memory and maps them directly into the GPU’s page tables.
- Allows the GPU's onboard DMA engines to copy weights directly across the PCIe bus, bypassing the host CPU.
- System Locked Memory is locked in Physical RAM but lacks DMA acceleration.
- CUDA Pinned Memory achieves maximum bandwidth with direct PCIe DMA transfer bypassing the CPU.
