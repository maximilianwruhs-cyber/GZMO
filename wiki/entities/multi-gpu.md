---
type: entity
title: multi-GPU
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# multi-GPU

Type: CONCEPT

## From [[drive-research-llamacpp-optimization-blueprint-micro04|drive-research-llamacpp-optimization-blueprint-micro04]] (2026-06-09)
- llama.cpp supports multi-GPU setups
- Performance breakthroughs for multi-GPU setups exist

## From [[optimizing-nvidia-blackwell-sm120-part3-micro06|optimizing-nvidia-blackwell-sm120-part3-micro06]] (2026-06-09)
- It involves distributing model weights across asymmetric graphics cards.
- It requires uniform hardware topologies.
- It can lead to synchronization failures or data corruption if not configured properly.
