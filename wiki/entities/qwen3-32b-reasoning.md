---
type: entity
title: Qwen3-32B (Reasoning)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Qwen3-32B (Reasoning)

Type: BOOK

## From [[drive-research-32gb-vram-ai-reasoning-models-micro02|drive-research-32gb-vram-ai-reasoning-models-micro02]] (2026-06-09)
- Core dense model contains 32.8 billion total parameters.
- Architecturally constructed with 64 layers and a hidden dimension size of 5,120.
- Natively implements Grouped-Query Attention (GQA).
- Utilizes hardware-efficient SwiGLU activation functions.
- Utilizes Rotary Positional Embeddings (RoPE).
- Utilizes an advanced RMSNorm pre-normalization structure.
- Developers entirely eliminated the QKV-bias utilized in prior Qwen2 generations.
- Natively supports a robust 32,768-token context, scaling to 131,072 tokens utilizing YaRN.
- The Reasoning variant achieves a GPQA score of 66.8%.
- The Reasoning variant secures an AIME 2025 score of 73.0%.
- When deployed on TensorRT-LLM frameworks, throughput blazes at between 101.5 to 180 tokens per second.
- Demands the absolute highest tier of logical fidelity, mathematical theorem proving, and complex, multi-step coding.
- Represents the maximum intelligence density physically achievable within 32 GB of VRAM.
- Scores consistently higher on GPQA and AIME 2025 than older 70B parameter models.
