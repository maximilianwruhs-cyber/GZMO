---
type: entity
title: Maximum Marginal Relevance (MMR)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Maximum Marginal Relevance (MMR)

Type: CONCEPT

## From [[openclaw-part1-micro06|openclaw-part1-micro06]] (2026-06-09)
- Algorithm used in the Deep Phase of the Dreaming-Engine.
- Balances relevance against redundancy to ensure diversity in stored concepts.
- Formula: λ × Relevance - (1-λ) × Maximum_Similarity_to_Selected.
