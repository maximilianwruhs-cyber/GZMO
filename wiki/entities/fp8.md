---
type: entity
title: FP8
created: 2026-06-08
updated: 2026-06-09
sources: 6
tags: []
status: draft
gzmo_synthetic: true
---






# FP8

Type: CONCEPT

## From [[drive-research-blackwell-sm120-gemm-optimization-guide|drive-research-blackwell-sm120-gemm-optimization-guide]] (2026-06-08)
- SM120 FP8 GEMM
- SM120 blockwise FP8 GEMM

## From [[drive-research-flashinfer-moe-fp4-jit-error|drive-research-flashinfer-moe-fp4-jit-error]] (2026-06-08)
- Used in MoE models
- Autotuner crashes when profiling tactics for pure FP8 MoE models on Blackwell

## From [[drive-research-32gb-vram-ai-reasoning-models-micro01|drive-research-32gb-vram-ai-reasoning-models-micro01]] (2026-06-09)
- An 8-bit quantization format
- Used as a production standard in enterprise ecosystems
- Leverages native FP8 Tensor Cores of Blackwell architecture

## From [[drive-research-32gb-vram-ai-reasoning-models-micro03|drive-research-32gb-vram-ai-reasoning-models-micro03]] (2026-06-09)
- Blackwell optimization used by TensorRT-LLM.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro01|drive-research-linux-gaming-and-ai-build-guide-micro01]] (2026-06-09)
- Operations relying on FP8 are accelerated by Tensor Cores.
- FP8 is a numerical format used in AI computations.

## From [[drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01|drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01]] (2026-06-09)
- A Tensor Core format supported by SM120 and SM121.
- TensorRT exhibits performance regressions for FP8 models on GB200 configurations.
- QuartzNet networks experience performance regressions on SM120 Blackwell platforms.
