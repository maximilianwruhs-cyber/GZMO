---
type: entity
title: HIP
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# HIP

Type: SYSTEM

## From [drive-research-llamacpp-gpu-memory-reporting-bug](/entities/drive-research-llamacpp-gpu-memory-reporting-bug.md) (2026-06-08)
- An active backend queried by llama_params_fit.
- Systems running HIP backends can experience dynamic VRAM accumulation.

## From [optimizing-nvidia-blackwell-sm120-part2-micro04](/entities/optimizing-nvidia-blackwell-sm120-part2-micro04.md) (2026-06-10)
- Compiler used for ROCm.
- Acts as a direct translation of the CUDA codebase.

## From [optimizing-nvidia-blackwell-sm120-part3-micro03](/entities/optimizing-nvidia-blackwell-sm120-part3-micro03.md) (2026-06-10)
- An active backend queried by the engine to determine memory capacity.
- Used in ROCm/HIP builds where UMA memory detection was bypassed to enable safe memory limit queries.
