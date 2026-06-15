---
type: entity
title: Flash Attention
created: 2026-06-09
updated: 2026-06-10
sources: 8
tags: []
status: draft
gzmo_synthetic: true
---









# Flash Attention

Type: CONCEPT

## From [[building-a-private-local-ai-development-environmen-micro01|building-a-private-local-ai-development-environmen-micro01]] (2026-06-09)
- Must be activated in LM Studio
- Allows Continue and an agent to access the model simultaneously without re-calculating context

## From [[building-a-private-local-ai-development-environmen-micro02|building-a-private-local-ai-development-environmen-micro02]] (2026-06-09)
- Massively accelerates attention calculation for long contexts.
- Can be enabled in LM Studio hardware acceleration settings.

## From [[drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01|drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01]] (2026-06-09)
- Kernels that can be enabled to drastically lower memory bandwidth demands during high-context prefill.
- Configured via the --flash-attn parameter.

## From [[drive-research-llama-bench-performance-benchmarking-tool-micro01|drive-research-llama-bench-performance-benchmarking-tool-micro01]] (2026-06-09)
- Optimized implementations are included in ROCm.
- Absence of optimized Flash Attention kernels in the standard Vulkan backend becomes a bottleneck at deep context lengths.

## From [[drive-research-llama-bench-performance-benchmarking-tool-micro02|drive-research-llama-bench-performance-benchmarking-tool-micro02]] (2026-06-09)
- Kernels are optimized in the ROCm backend.
- Used for context lengths exceeding 10,000 tokens.

## From [[drive-research-llamacpp-optimization-blueprint-micro03|drive-research-llamacpp-optimization-blueprint-micro03]] (2026-06-09)
- Enables memory-efficient attention computation.
- Mandatory for contexts >= 8k.
- Radically reduces the KV cache size by preventing N x N matrix materialization.

## From [[optimizing-nvidia-blackwell-sm120-part1-micro01|optimizing-nvidia-blackwell-sm120-part1-micro01]] (2026-06-10)
- An algorithm that reduces the VRAM footprint of the Key-Value (KV) cache.
- Fuses operations to prevent materialization of the attention matrix in HBM.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro05|optimizing-nvidia-blackwell-sm120-part2-micro05]] (2026-06-10)
- Optimized kernels available in the ROCm backend
