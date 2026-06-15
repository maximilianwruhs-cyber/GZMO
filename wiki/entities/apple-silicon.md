---
type: entity
title: Apple Silicon
created: 2026-06-08
updated: 2026-06-10
sources: 7
tags: []
status: draft
gzmo_synthetic: true
---







# Apple Silicon

Type: SYSTEM

## From [[drive-research-llamacpp-gpu-memory-reporting-bug|drive-research-llamacpp-gpu-memory-reporting-bug]] (2026-06-08)
- A system with Unified Memory Architecture.
- UMA detection logic works correctly on Apple Silicon.
- Memory addressing differs from discrete AMD GPUs on macOS.

## From [[drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01|drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01]] (2026-06-09)
- A unified memory architecture.
- RAM is shared dynamically between host execution processes and model compute kernels.
- Allocating excessive CPU cache can cause memory thrashing and OOM failures.

## From [[drive-research-llama-bench-performance-benchmarking-tool-micro01|drive-research-llama-bench-performance-benchmarking-tool-micro01]] (2026-06-09)
- Utilizes an architecture centered around wide LPDDR5/LPDDR5x unified memory buses.
- Enables high token generation speeds on consumer hardware.
- Prompt processing speeds are often limited by raw compute performance compared to dedicated CUDA pipelines.

## From [[drive-research-llm-inference-engine-audit-2026-micro02|drive-research-llm-inference-engine-audit-2026-micro02]] (2026-06-09)
- MLX provides immediate performance leaps on it when natively leveraged by Ollama.
- MLX is the definitive choice for maximum local throughput on it.

## From [[drive-research-llm-inference-engine-audit-2026-micro03|drive-research-llm-inference-engine-audit-2026-micro03]] (2026-06-09)
- Apple M5 GPU is used for exploring LLMs with MLX.
- Apple M5 Pro and M5 Max are debuted.
- Apple Silicon is mentioned in relation to Ollama and MLX.
- Used for exploring LLMs with MLX.
- Debuted to supercharge demanding pro workflows.
- Ollama is powered by MLX on it in preview.

## From [[optimizing-nvidia-blackwell-sm120-part2-micro04|optimizing-nvidia-blackwell-sm120-part2-micro04]] (2026-06-10)
- Architecture utilizing wide LPDDR5/LPDDR5x unified memory buses.
- High memory bandwidth enables excellent sequential decode performance.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro03|optimizing-nvidia-blackwell-sm120-part3-micro03]] (2026-06-10)
- Features Unified Memory Architecture (UMA) where system RAM and GPU memory share a single physical pool.
