---
type: entity
title: ROCm
created: 2026-06-08
updated: 2026-06-10
sources: 12
tags: []
status: draft
gzmo_synthetic: true
---












# ROCm

Type: SYSTEM

## From [[drive-research-llamacpp-gpu-memory-reporting-bug|drive-research-llamacpp-gpu-memory-reporting-bug]] (2026-06-08)
- Multi-GPU systems running ROCm backends can experience dynamic VRAM accumulation.
- Avoid defining conflicting environment variables like ROCR_VISIBLE_DEVICES and HIP_VISIBLE_DEVICES.

## From [[drive-research-to-product-engineering-leadership|drive-research-to-product-engineering-leadership]] (2026-06-08)
- AMD's GPU computing platform.
- Can be probed dynamically or via file stat check.

## From [[drive-research-cuda-graph-capture-failure-workarounds-micro02|drive-research-cuda-graph-capture-failure-workarounds-micro02]] (2026-06-09)
- Backend where Gemma 4 MoE architectures can encounter prompt-parsing vulnerabilities.

## From [[drive-research-llama-bench-performance-benchmarking-tool-micro01|drive-research-llama-bench-performance-benchmarking-tool-micro01]] (2026-06-09)
- AMD's backend, primarily structured as a direct translation of CUDA via HIP compiler.
- Lacks low-level scheduling and vector register tuning for RDNA3 and RDNA4 architectures.
- Includes highly optimized Flash Attention implementations.

## From [[drive-research-llama-bench-performance-benchmarking-tool-micro02|drive-research-llama-bench-performance-benchmarking-tool-micro02]] (2026-06-09)
- Has optimized Flash Attention kernels.
- Is a more scalable choice for context lengths exceeding 10,000 tokens.

## From [[drive-research-llamacpp-optimization-blueprint-micro02|drive-research-llamacpp-optimization-blueprint-micro02]] (2026-06-09)
- Used for execution in favor of raw C/C++.

## From [[drive-research-llm-inference-engine-audit-2026-micro02|drive-research-llm-inference-engine-audit-2026-micro02]] (2026-06-09)
- Used by llama.cpp on AMD silicon.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro06|drive-research-linux-gaming-and-ai-build-guide-micro06]] (2026-06-10)
- AMD's GPU computing platform
- Version 7.1 mentioned

## From [[optimizing-nvidia-blackwell-sm120-part1-micro01|optimizing-nvidia-blackwell-sm120-part1-micro01]] (2026-06-10)
- An execution platform used alongside CUDA for hardware-proximate execution.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro04|optimizing-nvidia-blackwell-sm120-part2-micro04]] (2026-06-10)
- Backend for AMD hardware.
- Includes highly optimized Flash Attention (FA) implementations.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro05|optimizing-nvidia-blackwell-sm120-part2-micro05]] (2026-06-10)
- Provides optimized Flash Attention kernels for context lengths exceeding 10,000 tokens
- Used as a backend for AMD hardware

## From [[the-2026-linux-workstation-micro02|the-2026-linux-workstation-micro02]] (2026-06-10)
- AMD's Radeon Open Compute platform
- ROCm 7.1 released in early 2026
