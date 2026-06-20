---
type: entity
title: BPE
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# BPE

Type: TOOL

## From [drive-research-subword-tokenization-mitigates-llm-sparsity-micro02](/entities/drive-research-subword-tokenization-mitigates-llm-sparsity-micro02.md) (2026-06-09)
- Assigns single, indivisible tokens to highly frequent words.
- Utilizes longer combinations of tokens to construct infrequent words.
- Acts as an information-theoretic arbitrage mechanism.
- Effectively mitigates sparsity by segmenting rare words into subword units.
- Can lead to pathological segmentations that contradict linguistic rules.
- Is the most widespread algorithm for mitigating Zipfian sparsity.

## From [drive-research-subword-tokenization-mitigates-llm-sparsity-micro03](/entities/drive-research-subword-tokenization-mitigates-llm-sparsity-micro03.md) (2026-06-09)
- used for inference latency of 710 ms for a 124M parameter model
- resulted in an OOV rate of 0.15% on a 1B-token held-out set
- resulted in an OOV rate of 4.9% under a 3% character-level substitution noise simulation
