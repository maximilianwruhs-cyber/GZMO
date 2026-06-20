---
type: entity
title: WSL2
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# WSL2

Type: SYSTEM

## From [drive-research-flashinfer-moe-fp4-jit-error](/entities/drive-research-flashinfer-moe-fp4-jit-error.md) (2026-06-08)
- Windows Subsystem for Linux
- Requires version 2.7.0 or higher with updated dxgkrnl driver for CUDA graph captures
- Older versions can trigger system reboots

## From [drive-research-optimizing-qwen36-on-blackwell-gpus](/entities/drive-research-optimizing-qwen36-on-blackwell-gpus.md) (2026-06-08)
- WSL2 kernels older than version 2.7.0 are prone to paravirtualization driver resets and spontaneous reboots.
- The host system must be upgraded to WSL2 version 2.7.0 preview or newer for stable vLLM operation.
- vLLM under WSL2 introduces several configuration challenges.

## From [optimizing-nvidia-blackwell-sm120-part1-micro07](/entities/optimizing-nvidia-blackwell-sm120-part1-micro07.md) (2026-06-10)
- Windows Subsystem for Linux.
- Requires version 2.7.0 or higher for runtime stability on Blackwell hardware.
- Older versions (2.6.x) can trigger Kernel-Power 41 failures under CUDA graph capture pressure.
