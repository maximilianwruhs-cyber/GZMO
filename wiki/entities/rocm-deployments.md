---
type: entity
title: ROCm deployments
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# ROCm deployments

Type: SYSTEM

## From [[optimizing-nvidia-blackwell-sm120-part3-micro04|optimizing-nvidia-blackwell-sm120-part3-micro04]] (2026-06-09)
- Backend for multi-GPU systems.
- Can experience dynamic VRAM accumulation.
- Used in ROCm deployments.
- Avoid defining both ROCR_VISIBLE_DEVICES and HIP_VISIBLE_DEVICES.
- Use HIP_VISIBLE_DEVICES alone for stable GPU targeting.
