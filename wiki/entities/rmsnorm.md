---
type: entity
title: RMSNorm
created: 2026-06-08
updated: 2026-06-09
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# RMSNorm

Type: CONCEPT

## From [ai-research-part1](/entities/ai-research-part1.md) (2026-06-08)
- AttnRes introduces one RMSNorm per layer.
- Removing it degrades both Full AttnRes (1.743) and Block AttnRes (1.750).
- Prevents individual layers with naturally larger outputs from dominating the softmax.

## From [ai-research-part7](/entities/ai-research-part7.md) (2026-06-08)
- It is applied to the flattened input hidden matrix x'l.
- It imposes significant latency in mHC when operating on the high-dimensional hidden state.
- Its weight is absorbed in φl in the specialized mHC kernels.
- Stands for Root Mean Square Layer Normalization.
- Used as a Layer Norm Type in the DeepSeek-V3 models.
- Referenced in Zhang and Sennrich, 2019.

## From [ai-research-part6-micro01](/entities/ai-research-part6-micro01.md) (2026-06-09)
- A variant of Layer Normalization.
- Used in deep learning architectures.

## From [ai-research-part6-micro04](/entities/ai-research-part6-micro04.md) (2026-06-09)
- Layer Norm Type used in OLMo-1.3B
- epsilon = 1e-5

## From [architectures-and-optimizations-for-speculative-de-micro04](/entities/architectures-and-optimizations-for-speculative-de-micro04.md) (2026-06-09)
- Layer normalization used in Qwen2.5 models for stable layer normalization.
