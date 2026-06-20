---
type: entity
title: ngram-mod
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# ngram-mod

Type: CONCEPT

## From [architectures-and-optimizations-for-speculative-de-micro05](/entities/architectures-and-optimizations-for-speculative-de-micro05.md) (2026-06-09)
- Uses a rolling hash computed via a Linear Congruential Generator (LCG).
- Maintains a shared hash pool across all server slots.
- Optimal for long-session reasoning models that frequently repeat cognitive traces.
- Operates within a shared hash pool of approximately 16 MB.
- Offers constant memory complexity and can generate variable draft lengths.
