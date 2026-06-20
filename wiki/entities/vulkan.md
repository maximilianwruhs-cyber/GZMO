---
type: entity
title: Vulkan
created: 2026-06-08
updated: 2026-06-10
sources: 9
tags: []
status: draft
gzmo_synthetic: true
---









# Vulkan

Type: SYSTEM

## From [drive-research-llamacpp-gpu-memory-reporting-bug](/entities/drive-research-llamacpp-gpu-memory-reporting-bug.md) (2026-06-08)
- An active backend queried by llama_params_fit.

## From [drive-research-cuda-graph-capture-failure-workarounds-micro02](/entities/drive-research-cuda-graph-capture-failure-workarounds-micro02.md) (2026-06-09)
- Backends avoid a specific failure mode by falling back to a PEG-native formatting chat structure.
- Demonstrates that frontend prompt parsing can impact backend compute stability.

## From [drive-research-linux-gaming-and-ai-build-guide-micro05](/entities/drive-research-linux-gaming-and-ai-build-guide-micro05.md) (2026-06-09)
- Graphics API.
- DirectStorage APIs are mapped to it.

## From [drive-research-llama-bench-performance-benchmarking-tool-micro01](/entities/drive-research-llama-bench-performance-benchmarking-tool-micro01.md) (2026-06-09)
- Backend for AMD hardware.
- Running on the open-source Mesa RADV driver.
- Consistently outperforms AMD’s ROCm/HIP backend in token generation.
- Absence of optimized Flash Attention (FA) kernels becomes a bottleneck at deep contexts.

## From [dynamics-of-the-unpredictable-micro06](/entities/dynamics-of-the-unpredictable-micro06.md) (2026-06-09)
- A lower-level, platform-specific graphics backend.
- Abstracted over by wgpu.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro01](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro01.md) (2026-06-09)
- A compute backend for cross-platform GPU support.
- Used when compiling llama.cpp for cross-platform GPU support.

## From [optimizing-nvidia-blackwell-sm120-part2-micro04](/entities/optimizing-nvidia-blackwell-sm120-part2-micro04.md) (2026-06-10)
- Backend that outperforms ROCm/HIP in token generation on AMD hardware.
- Lacks optimized Flash Attention (FA) kernels for contexts exceeding 10,000 tokens.

## From [optimizing-nvidia-blackwell-sm120-part3-micro03](/entities/optimizing-nvidia-blackwell-sm120-part3-micro03.md) (2026-06-10)
- An active backend queried by the engine to determine memory capacity.

## From [the-2026-linux-workstation-micro04](/entities/the-2026-linux-workstation-micro04.md) (2026-06-10)
- Target for DirectX call translation
