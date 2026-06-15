---
type: entity
title: 'llama.cpp / ik_llama MoE Expert Offloading - Main Memory Bandwidth vs. PCIe Bandwidth : r/LocalLLaMA - Reddit'
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# llama.cpp / ik_llama MoE Expert Offloading - Main Memory Bandwidth vs. PCIe Bandwidth : r/LocalLLaMA - Reddit

Type: TOOL

## From [[drive-research-frankenmoe-merging-ai-models|drive-research-frankenmoe-merging-ai-models]] (2026-06-08)
- Enables CPU-GPU hybrid execution for environments where total VRAM is less than the model footprint.
- The --cpu-moe flag allows KV cache, attention layers, and shared experts to be offloaded to the GPU.
- Provides CPU-GPU hybrid execution for MoE models.
- Supports offloading KV cache, attention layers, and shared experts to GPU.
- Has a --cpu-moe flag for hybrid mode.
- A blog post on Clarifai about llama.cpp.
- Discusses local LLM inference and tuning.
- A Reddit discussion about llama.cpp MoE expert offloading.
- Compares memory bandwidths.
