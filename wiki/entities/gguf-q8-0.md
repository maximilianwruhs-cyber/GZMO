---
type: entity
title: GGUF Q8_0
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# GGUF Q8_0

Type: CONCEPT

## From [architectural-blueprints-for-sovereign-frankenmoe-part1](/entities/architectural-blueprints-for-sovereign-frankenmoe-part1.md) (2026-06-08)
- It is a quantization format.
- It has 8.5 bits per weight (bpw).
- A 3x7B FrankenMoE footprint is approximately 19 GB with this format.

## From [architectures-and-optimizations-for-speculative-de-micro04](/entities/architectures-and-optimizations-for-speculative-de-micro04.md) (2026-06-09)
- An 8-bit quantization format.
- Used for Qwen2.5-0.5B-Instruct to preserve probabilistic accuracy.
