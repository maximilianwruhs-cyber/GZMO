---
type: entity
title: Length-MAX
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Length-MAX

Type: TOOL

## From [[drive-research-subword-tokenization-mitigates-llm-sparsity-micro02|drive-research-subword-tokenization-mitigates-llm-sparsity-micro02]] (2026-06-09)
- Casts tokenization fundamentally as a length-weighted objective maximization problem.
- Minimizes the average number of tokens required per character.
- Preserves the underlying Zipfian structure of the token distribution.

## From [[drive-research-subword-tokenization-mitigates-llm-sparsity-micro03|drive-research-subword-tokenization-mitigates-llm-sparsity-micro03]] (2026-06-09)
- demonstrates exceptional performance gains
- reduced inference latency for a 124M parameter model from 710 ms to 446 ms compared to BPE
- maintains exceptionally low Out-Of-Vocabulary rates
