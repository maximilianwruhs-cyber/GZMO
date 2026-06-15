---
type: entity
title: CUDA toolkit
created: 2026-06-08
updated: 2026-06-10
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# CUDA toolkit

Type: TOOL

## From [[drive-research-du-hast-gesagt-part1|drive-research-du-hast-gesagt-part1]] (2026-06-08)
- Needed for NVIDIA GPU to run local AI models.
- Installed via `apt install -y nvidia-cuda-toolkit`.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro02|drive-research-linux-gaming-and-ai-build-guide-micro02]] (2026-06-09)
- Part of the massive AI stack.
- Must be strictly and hermetically containerized.
- NVIDIA Container Toolkit with Podman ensures CUDA runtime is seamlessly passed into the container.
- NVIDIA's CUDA 13.0 ecosystem makes AMD hardware fundamentally non-viable for the uncompromising AI practitioner.
- The NVIDIA Container Toolkit with Podman ensures that the CUDA runtime is seamlessly passed into the container.

## From [[the-2026-linux-workstation-micro03|the-2026-linux-workstation-micro03]] (2026-06-09)
- Part of the massive AI stack.
- Must be containerized.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro06|drive-research-linux-gaming-and-ai-build-guide-micro06]] (2026-06-10)
- NVIDIA's ecosystem
- Monopoly over TensorRT-LLM and FlashAttention 3
- Part of the AI stack
- Version 13.0 mentioned
