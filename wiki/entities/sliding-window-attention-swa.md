---
type: entity
title: Sliding Window Attention (SWA)
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Sliding Window Attention (SWA)

Type: CONCEPT

## From [[optimizing-nvidia-blackwell-sm120-part3-micro05|optimizing-nvidia-blackwell-sm120-part3-micro05]] (2026-06-09)
- Hybrid attention layers can trigger sequence-length-dependent crashes.
- Features shared key-value cache references.
- Shared KV cache boundaries can cross a critical threshold with long prompts.

## From [[prfaas-cross-datacenter-llm-serving-via-selective-micro02|prfaas-cross-datacenter-llm-serving-via-selective-micro02]] (2026-06-10)
- A hybrid-attention mechanism that maintains linear computation cost.
