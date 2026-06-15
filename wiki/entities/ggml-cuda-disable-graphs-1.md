---
type: entity
title: GGML_CUDA_DISABLE_GRAPHS=1
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# GGML_CUDA_DISABLE_GRAPHS=1

Type: SYSTEM

## From [[optimizing-nvidia-blackwell-sm120-part3-micro04|optimizing-nvidia-blackwell-sm120-part3-micro04]] (2026-06-09)
- Driver-level memory leaks can occur.
- Disabling CUDA graphs is a workaround for memory accumulation.
- Environment variable to disable CUDA graphs.
- Prevents memory leaks and system instability.
- Accumulation of unreleased CUDA graphs can cause progressive VRAM growth.
- Disabling via GGML_CUDA_DISABLE_GRAPHS=1 prevents memory exhaustion.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro05|optimizing-nvidia-blackwell-sm120-part3-micro05]] (2026-06-09)
- Environment variable to bypass graph capture.
- Forces the backend to bypass graph capture.
- Maintains long-term memory stability.
- Used to capture and replay static execution topologies.
- Capture and replay mechanism breaks down in RPC configurations.
- Accumulation of entries in the server-side framework leads to memory leaks.
