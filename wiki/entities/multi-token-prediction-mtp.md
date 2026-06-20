---
type: entity
title: Multi-Token Prediction (MTP)
created: 2026-06-08
updated: 2026-06-09
sources: 9
tags: []
status: draft
gzmo_synthetic: true
---










# Multi-Token Prediction (MTP)

Type: CONCEPT

## From [drive-research-agentic-workflows-fastest-best-models](/entities/drive-research-agentic-workflows-fastest-best-models.md) (2026-06-08)
- An inference acceleration technique.
- Allows the model to predict multiple future tokens simultaneously.
- Acts as its own drafter without requiring a standalone secondary model.
- Yields significant generation speedups.
- Provides a proportionally larger boost to heavier quantizations.
- Enabled in llama.cpp.
- Bypasses need for external draft models.
- Delivers acceleration in throughput.

## From [drive-research-llamacpp-gpu-memory-reporting-bug](/entities/drive-research-llamacpp-gpu-memory-reporting-bug.md) (2026-06-08)
- Integrated into models like Qwen.
- Requires a secondary, built-in draft context for speculative token generation.

## From [drive-research-optimizing-qwen36-on-blackwell-gpus](/entities/drive-research-optimizing-qwen36-on-blackwell-gpus.md) (2026-06-08)
- Modifies the standard sequential decoding process by allowing the model to draft multiple future tokens in parallel.
- Accelerates generation speeds by 1.4x to 2.2x.
- Requires the allocation of additional tracking heads, increasing memory consumption.

## From [building-a-private-local-ai-development-environmen-micro06](/entities/building-a-private-local-ai-development-environmen-micro06.md) (2026-06-09)
- A type of 'Built-in' Drafter.
- Popularized by DeepSeek-V3 and DeepSeek-R1 architectures.
- The core model is trained with extra 'prediction heads'.
- Predicts the next 2 to 3 sequential words simultaneously.

## From [optimizing-nvidia-blackwell-sm120-part1-micro04](/entities/optimizing-nvidia-blackwell-sm120-part1-micro04.md) (2026-06-09)
- Speculative draft heads are trained and optimized to operate on native FP4 activation distributions.
- When using Marlin, dequantization to FP16 introduces minor numerical deviations in output activations.

## From [optimizing-nvidia-blackwell-sm120-part3-micro05](/entities/optimizing-nvidia-blackwell-sm120-part3-micro05.md) (2026-06-09)
- Speculatively decoded architectures using MTP options show progressive memory inflation.
- Can lead to out-of-memory crashes during graph execution update.
- Does not properly recycle execution contexts across speculative verification steps.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro06](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro06.md) (2026-06-09)
- Natively train models to predict several future tokens at once
- Completely removing the need for separate drafting infrastructure

## From [the-architecture-of-speculative-decoding-and-infer-part2-micro02](/entities/the-architecture-of-speculative-decoding-and-infer-part2-micro02.md) (2026-06-09)
- A training objective pioneered by DeepSeek-V3.
- Optimizes the model to predict multiple future tokens simultaneously.
- An auxiliary-loss-free strategy.

## From [ultimate-local-ai-development-stack-for-vscodium-micro02](/entities/ultimate-local-ai-development-stack-for-vscodium-micro02.md) (2026-06-09)
- An evolution of Speculative Decoding where prediction heads are built into the core model.
- Popularized by DeepSeek-V3 and DeepSeek-R1 architectures.
