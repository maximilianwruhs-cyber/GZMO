---
type: entity
title: CPU
created: 2026-06-08
updated: 2026-06-09
sources: 8
tags: []
status: draft
gzmo_synthetic: true
---








# CPU

Type: SYSTEM

## From [drive-research-advanced-inference-acceleration](/entities/drive-research-advanced-inference-acceleration.md) (2026-06-08)
- Used when models are offloaded from GPU.
- Offloading draft models to CPU degrades the time ratio 'c'.

## From [drive-research-du-hast-gesagt-part1](/entities/drive-research-du-hast-gesagt-part1.md) (2026-06-08)
- Runs Matmul-free 1.58-bit models.
- Handles ultra-fast autocomplete and execution tasks.
- Used by Continue.dev and Aider.

## From [drive-research-llmlingua-cpu-leistung-und-leistungstests](/entities/drive-research-llmlingua-cpu-leistung-und-leistungstests.md) (2026-06-08)
- The Hermes architecture requires the compression backend to run locally on a CPU.
- LLMLingua-2 models are suitable for CPU operation.
- CPU execution incurs performance penalties compared to GPU.

## From [drive-research-linux-gaming-and-ai-build-guide-micro01](/entities/drive-research-linux-gaming-and-ai-build-guide-micro01.md) (2026-06-09)
- Requirements are bifurcated: gaming demands single-core frequency and L3 cache.
- AI dataset preprocessing demands multi-core throughput.
- The AMD Ryzen 9 9950X3D is presented as the optimal processor.

## From [drive-research-linux-gaming-and-ai-build-guide-micro03](/entities/drive-research-linux-gaming-and-ai-build-guide-micro03.md) (2026-06-09)
- Best CPUs for 2026 tested
- Comparison between AMD Ryzen and Intel Core
- Comparison between AMD Zen 6 and Intel Nova Lake-S

## From [drive-research-llamacpp-optimization-blueprint-micro02](/entities/drive-research-llamacpp-optimization-blueprint-micro02.md) (2026-06-09)
- The host CPU remains responsible for memory orchestration, prompt parsing, sequence routing, and serving as the fallback compute device.
- Over-subscribing threads destroys performance due to context switching and cache thrashing.
- The -t (threads) command sets the number of CPU threads used during generation.

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro01](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro01.md) (2026-06-09)
- Used for offloading draft models when VRAM is insufficient.
- Offloading to CPU catastrophically degrades the time ratio 'c', making speculative configuration slower.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro05.md) (2026-06-09)
- Real-time performance monitoring is facilitated by powerstat.
