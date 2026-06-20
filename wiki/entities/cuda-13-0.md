---
type: entity
title: CUDA 13.0
created: 2026-06-08
updated: 2026-06-10
sources: 6
tags: []
status: draft
gzmo_synthetic: true
---






# CUDA 13.0

Type: SYSTEM

## From [drive-research-flashinfer-moe-fp4-jit-error](/entities/drive-research-flashinfer-moe-fp4-jit-error.md) (2026-06-08)
- Introduced compute_120f target flag
- Resolves compilation target suffix rift

## From [drive-research-optimizing-qwen36-on-blackwell-gpus](/entities/drive-research-optimizing-qwen36-on-blackwell-gpus.md) (2026-06-08)
- Required for deploying NVFP4, as default PyTorch toolkits do not support SM 12.0.
- Requires installing CUDA 13.0 nightly packages alongside the nvidia-cuda-nvcc compiler wheel.
- Enables the SM 12.0 architecture flag (compute_120f).

## From [drive-research-linux-gaming-and-ai-build-guide-micro02](/entities/drive-research-linux-gaming-and-ai-build-guide-micro02.md) (2026-06-09)
- NVIDIA's ecosystem.
- Exclusive monopoly over critical inference infrastructure like TensorRT-LLM and FlashAttention 3.
- Makes AMD hardware fundamentally non-viable for uncompromising AI practitioners.

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- Introduced the family-specific compiler target compute_120f.
- TensorRT exhibited performance regressions when compiled under this version.

## From [the-2026-linux-workstation-micro03](/entities/the-2026-linux-workstation-micro03.md) (2026-06-09)
- NVIDIA's ecosystem.
- Exclusive monopoly over critical inference infrastructure.

## From [the-2026-linux-workstation-micro04](/entities/the-2026-linux-workstation-micro04.md) (2026-06-10)
- Part of NVIDIA's AI software ecosystem
