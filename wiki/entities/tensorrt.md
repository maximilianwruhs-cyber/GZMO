---
type: entity
title: TensorRT
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# TensorRT

Type: SYSTEM

## From [drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01](/entities/drive-research-optimizing-cuda-performance-with-fp4-fp6-micro01.md) (2026-06-09)
- A library exhibiting performance regressions when compiled under CUDA 13.0.
- Exhibits up to a 24% performance regression for ResNext-50 FP8 models on GB200 configurations compared to CUDA 12.9.
- Shows a 40% throughput regression in multi-head attention blocks for Vision Transformer (ViT) architectures.
- FP8 QuartzNet networks experience up to a 77% performance regression on SM120 Blackwell platforms under TensorRT 10.13.2.
