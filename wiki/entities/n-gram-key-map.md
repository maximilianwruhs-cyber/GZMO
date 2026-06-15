---
type: entity
title: N-Gram Key Map
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# N-Gram Key Map

Type: CONCEPT

## From [[architectures-and-optimizations-for-speculative-de-micro05|architectures-and-optimizations-for-speculative-de-micro05]] (2026-06-09)
- A speculation type in llama.cpp (--spec-type ngram-map-k).
- Advances simple lookup by maintaining an internal hash-map.
- Tracks current n-gram and monitors subsequent continuations.
