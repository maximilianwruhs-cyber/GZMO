---
type: entity
title: FP4
created: 2026-06-08
updated: 2026-06-10
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# FP4

Type: CONCEPT

## From [drive-research-blackwell-sm120-gemm-optimization-guide](/entities/drive-research-blackwell-sm120-gemm-optimization-guide.md) (2026-06-08)
- A 4-bit floating-point data format.
- Used in post-training and inference pipelines.
- Has extreme dynamic range limitations.
- Native FP4
- FP4 E2M1

## From [drive-research-32gb-vram-ai-reasoning-models-micro03](/entities/drive-research-32gb-vram-ai-reasoning-models-micro03.md) (2026-06-09)
- Blackwell optimization used by TensorRT-LLM.

## From [drive-research-marlin-baseline-for-early-deployments-micro02](/entities/drive-research-marlin-baseline-for-early-deployments-micro02.md) (2026-06-09)
- A numerical format.
- Native path can be broken on SM120.
- Requires specific configurations for stable execution.

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Native FP4 pipelines are plagued by broken TMA structures, compiler bugs, and shared memory overflows on SM120.
- Marlin is capable of executing FP16 x FP4 operations.
- Speculative MTP draft heads are trained to operate on native FP4 activation distributions.

## From [optimizing-nvidia-blackwell-sm120-part1-micro06](/entities/optimizing-nvidia-blackwell-sm120-part1-micro06.md) (2026-06-10)
- 4-bit floating-point representation.
