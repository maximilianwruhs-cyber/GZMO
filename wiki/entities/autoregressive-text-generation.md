---
type: entity
title: Autoregressive Text Generation
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Autoregressive Text Generation

Type: CONCEPT

## From [[drive-research-erbandbreite-und-latenzengp-sse|drive-research-erbandbreite-und-latenzengpässe]] (2026-06-08)
- Transformer-based architectures generate text sequentially, processing a single token per forward pass.
- Requires loading entire parameter weight matrix and KV cache during each step.
- Low computational intensity leads to data starvation for AI accelerators.
