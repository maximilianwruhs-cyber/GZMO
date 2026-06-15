---
type: entity
title: Model Runner V2
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Model Runner V2

Type: SYSTEM

## From [[drive-research-hidden-mode-technical-analysis-and-configurati|drive-research-hidden-mode-technical-analysis-and-configurati]] (2026-06-08)
- Explicitly enabled via VLLM_USE_V2_MODEL_RUNNER=1.
- Uses GPU-native Triton kernels and asynchronous scheduling to improve throughput.
- An optimization within vLLM for model serving.
