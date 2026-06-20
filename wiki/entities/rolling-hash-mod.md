---
type: entity
title: Rolling Hash (Mod)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Rolling Hash (Mod)

Type: CONCEPT

## From [architectures-and-optimizations-for-speculative-de-micro05](/entities/architectures-and-optimizations-for-speculative-de-micro05.md) (2026-06-09)
- A speculation type in llama.cpp (--spec-type ngram-mod).
- Utilizes a rolling hash computed via a Linear Congruential Generator (LCG).
- Maintains a shared hash pool across all server slots.
