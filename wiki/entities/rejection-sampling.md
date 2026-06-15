---
type: entity
title: Rejection Sampling
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Rejection Sampling

Type: CONCEPT

## From [[drive-research-erbandbreite-und-latenzengp-sse|drive-research-erbandbreite-und-latenzengpässe]] (2026-06-08)
- A token-level verification algorithm used in speculative decoding.
- Ensures speedup does not compromise generative quality or accuracy.
- Acceptance logic compares probabilities assigned by draft and target models.

## From [[the-architecture-of-speculative-decoding-and-infer-part2-micro01|the-architecture-of-speculative-decoding-and-infer-part2-micro01]] (2026-06-09)
- A rigorous token-level verification algorithm.
- Ensures speedup does not compromise generative quality.
- Used in the acceptance logic for proposed tokens.
