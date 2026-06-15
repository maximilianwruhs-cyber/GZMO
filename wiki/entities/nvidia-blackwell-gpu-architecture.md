---
type: entity
title: NVIDIA Blackwell GPU architecture
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# NVIDIA Blackwell GPU architecture

Type: SYSTEM

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- A target architecture for high-throughput sub-byte quantized inference.
- Requires CUDA 13.1 runtime compilation environment and optimized low-level kernel templates for FP4/FP6.
- Exhibits unique compilation gaps, synchronization hazards, and memory alignment constraints for consumer/desktop variants (SM120, SM121) compared to enterprise datacenter (SM100) platforms.
- Demands a systematic combination of family-specific compiler targets, explicit PTX barrier controls, and hardware-enforced memory alignment patches for deployment.
