---
type: entity
title: Hugging Face Transformers
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Hugging Face Transformers

Type: SYSTEM

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro06|the-architecture-of-speculative-decoding-and-infer-part1-micro06]] (2026-06-09)
- Community-developed turboquant Python package provides a drop-in TurboQuantCache class that replaces standard Hugging Face dynamic caches
- Practitioners utilize the assistant_model parameter within the .generate() API for speculative drafting
