---
type: entity
title: Llama 4 Scout
created: 2026-06-09
updated: 2026-06-10
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---






# Llama 4 Scout

Type: BOOK

## From [drive-research-32gb-vram-ai-reasoning-models-micro02](/entities/drive-research-32gb-vram-ai-reasoning-models-micro02.md) (2026-06-09)
- Smallest model in Meta's primary Llama 4 release.
- Marketed as an efficient model with only 17 billion active parameters per token, utilizing 16 distinct experts.
- Total parameter count is a massive 109B.
- Consumes roughly 55 GB of VRAM even at INT4 quantization.
- Requires offloading over 40% of the model to system RAM on a single 32 GB GPU.
- Features an unparalleled 10 million token context window.
- Has excellent benchmark performance (AIME 14.0%, MATH 500 84.4%).
- Functionally unsuited for pure 32 GB local deployment.
- Aggressively embraces MoE designs.

## From [drive-research-linux-gaming-and-ai-build-guide-micro01](/entities/drive-research-linux-gaming-and-ai-build-guide-micro01.md) (2026-06-09)
- A MoE model with 109B total parameters and 17B active.
- Quantized to INT4, it still requires roughly 55GB of VRAM.
- Part of the Llama 4 family.

## From [drive-research-linux-gaming-and-ai-build-guide-micro05](/entities/drive-research-linux-gaming-and-ai-build-guide-micro05.md) (2026-06-09)
- Mixture-of-Experts (MoE) model.
- 109B total parameters, 17B active.
- Requires roughly 55GB of VRAM when quantized to INT4.
- Family of models by Meta.
- Relies heavily on Mixture-of-Experts (MoE) architectures.
- Expected in 2026.

## From [the-2026-linux-workstation-micro02](/entities/the-2026-linux-workstation-micro02.md) (2026-06-10)
- 109B total parameters, 17B active

## From [the-architecture-of-speculative-decoding-and-infer-part1-micro03](/entities/the-architecture-of-speculative-decoding-and-infer-part1-micro03.md) (2026-06-10)
- Developed by Meta.
- A Mixture-of-Experts (MoE) model with 109B total parameters.
- Features 16 distinct experts.
