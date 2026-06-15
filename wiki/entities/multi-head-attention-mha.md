---
type: entity
title: Multi-Head Attention (MHA)
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Multi-Head Attention (MHA)

Type: CONCEPT

## From [[drive-research-agentic-workflows-fastest-best-models|drive-research-agentic-workflows-fastest-best-models]] (2026-06-08)
- Leads to catastrophic memory scaling at long contexts.
- Number of KV heads equals total number of query heads.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro06|the-architecture-of-speculative-decoding-and-infer-part1-micro06]] (2026-06-09)
- TurboQuant was explicitly designed for standard MHA transformers
- Utilize Euclidean geometric mappings

## From [[drive-research-research-process-steps-micro03|drive-research-research-process-steps-micro03]] (2026-06-10)
- A mechanism where memory footprint is calculated using transformer layers, query heads, head dimension, sequence length, batch size, and element precision.
