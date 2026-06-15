---
type: entity
title: Draft Model Selection Strategies
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Draft Model Selection Strategies

Type: CONCEPT

## From [[drive-research-erbandbreite-und-latenzengp-sse|drive-research-erbandbreite-und-latenzengpässe]] (2026-06-08)
- Critical variable dictating overall inference system throughput.
- Parameter ratio between draft and target models is key.
- Draft model size impacts autoregressive generation latency and output distribution divergence.
- explores multiple potential paths simultaneously
- requires independent, synchronized KV caches
- must be significantly smaller than target models for optimal speedup
- if not heavily exposed to specific sub-domains, its conditional token distribution will diverge from the target model
- enters high-entropy guessing states for underrepresented languages or creative prose
- can be a smaller variant of the target model
