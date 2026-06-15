---
type: entity
title: Mamba
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Mamba

Type: SYSTEM

## From [[drive-research-speicherbandbreiten-engpass-memory-wall|drive-research-speicherbandbreiten-engpass-memory-wall]] (2026-06-08)
- A State Space Model (SSM).
- TurboQuant does not compress hidden states in SSMs like Mamba.
- If an SSM is used as a draft model, TurboQuant must be isolated to the Transformer-based target model.

## From [[the-architecture-of-speculative-decoding-and-infer-part1-micro06|the-architecture-of-speculative-decoding-and-infer-part1-micro06]] (2026-06-09)
- Example of a State Space Model (SSM)
- Has constant memory complexity during inference
