---
type: entity
title: llama_params_fit
created: 2026-06-08
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# llama_params_fit

Type: TOOL

## From [[drive-research-llamacpp-gpu-memory-reporting-bug|drive-research-llamacpp-gpu-memory-reporting-bug]] (2026-06-08)
- The automatic parameter-fitting module.
- Queries active backends to determine memory capacity.
- Initiates a deterministic, multi-stage downscaling pipeline if memory is exceeded.

## From [[optimizing-nvidia-blackwell-sm120-part3-micro03|optimizing-nvidia-blackwell-sm120-part3-micro03]] (2026-06-10)
- Acts as a pre-allocation constraint-satisfaction solver.
- Queries active backends to determine physical and free memory capacity.
- Initiates a multi-stage downscaling pipeline if projected memory use exceeds hardware capacity.
- Command-line argument that controls the automatic parameter-fitting subsystem.
- Default value is 'on'.
