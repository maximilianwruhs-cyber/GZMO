---
type: entity
title: Qwen3-32B
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Qwen3-32B

Type: SYSTEM

## From [drive-research-32gb-vram-ai-reasoning-models-micro01](/entities/drive-research-32gb-vram-ai-reasoning-models-micro01.md) (2026-06-09)
- A leading reasoning model for 32 GB VRAM CUDA environments
- Part of the 26B to 32B parameter tier
- Can be deployed with 8-bit (FP8) or custom mixed-quantization formats

## From [drive-research-32gb-vram-ai-reasoning-models-micro02](/entities/drive-research-32gb-vram-ai-reasoning-models-micro02.md) (2026-06-09)
- Core dense model contains 32.8 billion total parameters.
- Architecturally constructed with 64 layers and a hidden dimension size of 5,120.
- Natively implements Grouped-Query Attention (GQA).
- Utilizes hardware-efficient SwiGLU activation functions, Rotary Positional Embeddings (RoPE), and an advanced RMSNorm pre-normalization structure.
- Developers entirely eliminated the QKV-bias utilized in prior Qwen2 generations.
- Natively supports a robust 32,768-token context, which scales algorithmically to an immense 131,072 tokens utilizing the YaRN methodology.
- The specific Qwen3 32B (Reasoning) variant achieves a GPQA score of 66.8% and an AIME 2025 score of 73.0%.
- When deployed on highly optimized TensorRT-LLM frameworks, throughput blazes at between 101.5 to 180 tokens per second.
