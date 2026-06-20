---
type: entity
title: Model Runner V2 (MRV2)
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Model Runner V2 (MRV2)

Type: CONCEPT

## From [architectural-blueprints-for-sovereign-frankenmoe-part1](/entities/architectural-blueprints-for-sovereign-frankenmoe-part1.md) (2026-06-08)
- Explicitly enabled via VLLM_USE_V2_MODEL_RUNNER=1.
- Uses GPU-native Triton kernels and asynchronous scheduling.
- Aims to improve throughput in vLLM.
- Aims to improve throughput.
- It is explicitly enabled via VLLM_USE_V2_MODEL_RUNNER=1.
- It uses GPU-native Triton kernels and asynchronous scheduling.
- It works alongside Chunked Prefill to improve token-per-second throughput.

## From [drive-research-optimizing-qwen36-on-blackwell-gpus](/entities/drive-research-optimizing-qwen36-on-blackwell-gpus.md) (2026-06-08)
- Used by vLLM to launch the server.
- Uses native Triton kernels and asynchronous scheduling to improve inference throughput.
- Enabled by setting the environment variable VLLM_USE_V2_MODEL_RUNNER=1.
