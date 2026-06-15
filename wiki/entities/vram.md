---
type: entity
title: VRAM
created: 2026-06-08
updated: 2026-06-09
sources: 9
tags: []
status: draft
gzmo_synthetic: true
---









# VRAM

Type: CONCEPT

## From [[drive-research-advanced-inference-acceleration|drive-research-advanced-inference-acceleration]] (2026-06-08)
- Video RAM, a type of computer memory.
- Models need to reside in VRAM for efficient speculative decoding.
- Scaling to larger models like 27B target with 9B draft exceeds 8GB VRAM capacity.

## From [[drive-research-hermes-anthropic-openrouter-cache-investigation|drive-research-hermes-anthropic-openrouter-cache-investigation]] (2026-06-08)
- Video Random Access Memory.
- Manages the KV-Cache during inference.
- Memory saturation is a permanent operational threat.

## From [[drive-research-linux-gaming-and-ai-build-guide-micro01|drive-research-linux-gaming-and-ai-build-guide-micro01]] (2026-06-09)
- The absolute hard limit that defines which AI models can operate efficiently.
- 32GB of VRAM is the current threshold for flagship capabilities.
- Context window expansion drastically alters the VRAM calculus.

## From [[drive-research-llamacpp-optimization-blueprint-micro02|drive-research-llamacpp-optimization-blueprint-micro02]] (2026-06-09)
- Flash Attention heavily reduces the VRAM footprint of the Key-Value (KV) cache at high context sizes.
- Pushing all layers to the GPU is the primary objective for VRAM utilization.
- If the model footprint exceeds the aggregate VRAM capacity, layers remain on the host CPU and system RAM.

## From [[drive-research-llamacpp-optimization-blueprint-micro03|drive-research-llamacpp-optimization-blueprint-micro03]] (2026-06-09)
- Physical micro-batch size must be adjusted downward if VRAM memory spikes occur.
- Speculative decoding is used when the GPU is bound by memory bandwidth rather than compute availability.
- Prompt Lookup Decoding is a zero-VRAM paradigm.

## From [[drive-research-llm-inference-engine-audit-2026-micro02|drive-research-llm-inference-engine-audit-2026-micro02]] (2026-06-09)
- Up to 120GB of standard system RAM can be allocated directly to it on Linux systems running Strix Halo.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro02|optimizing-nvidia-blackwell-sm120-part1-micro02]] (2026-06-09)
- Setting -c strictly to workload requirements preserves VRAM.
- Adjusting -ub downward prevents VRAM memory spikes during prompt processing.
- When the GPU is bound by memory bandwidth rather than compute availability, CUDA execution units lie dormant waiting for weights to arrive from VRAM over the memory bus.
- Draft models consume some VRAM.
- Prompt Lookup Decoding is a zero-VRAM paradigm.
- When the context window expands toward massive frontiers, the VRAM consumed by the Key and Value matrices rapidly eclipses the memory footprint of the static model weights.
- If VRAM is exhausted by the context window before the model weights, system architects must quantize the KV cache itself to preserve stability.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro04|optimizing-nvidia-blackwell-sm120-part3-micro04]] (2026-06-09)
- Dynamic accumulation can occur during generation.
- Occupancy exceeding 82% can lead to continuous growth.
- Configuring a safety margin helps keep memory use below leak threshold.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro01|the-architecture-of-speculative-decoding-and-infer-part1-micro01]] (2026-06-09)
- Video RAM, a critical hardware constraint for speculative decoding.
- Models and KV caches must fit within VRAM for efficient execution.
- Exceeding VRAM capacity leads to performance degradation or impossibility of execution.
