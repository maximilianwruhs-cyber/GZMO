---
type: entity
title: Quantization Aware Distillation (QAD)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Quantization Aware Distillation (QAD)

Type: CONCEPT

## From [drive-research-optimizing-qwen36-on-blackwell-gpus](/entities/drive-research-optimizing-qwen36-on-blackwell-gpus.md) (2026-06-08)
- Minimizes the gap between high-precision and low-precision outputs.
- Necessary for models with fewer than 600 billion parameters to avoid severe accuracy degradation with standard NVFP4 quantization.
- Checkpoints generated without QAD may exhibit high perplexity or corrupted outputs.
