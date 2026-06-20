---
type: entity
title: Vulkan environment variable
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Vulkan environment variable

Type: API

## From [drive-research-ubuntu-extreme-hardware-tuning-micro01](/entities/drive-research-ubuntu-extreme-hardware-tuning-micro01.md) (2026-06-09)
- Graphics API.
- Can have device selection conflicts in multi-GPU systems.
- Runtimes can be forced to target discrete GPUs using environment variables.
- Used to force Vulkan runtimes to target discrete GPUs.
- Example: `export VK_ICD_FILTERS=nvidia`.
