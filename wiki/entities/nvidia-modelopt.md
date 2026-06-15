---
type: entity
title: NVIDIA Modelopt
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# NVIDIA Modelopt

Type: TOOL

## From [[drive-research-optimizing-qwen36-on-blackwell-gpus|drive-research-optimizing-qwen36-on-blackwell-gpus]] (2026-06-08)
- Standard NVFP4 models compiled with this toolkit are not uniform 4-bit.
- By default, they preserve self-attention layers in high-precision BF16.
- Creates mixed-precision checkpoints.
