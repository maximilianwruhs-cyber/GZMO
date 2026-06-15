---
type: entity
title: Gemma 2 9B
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Gemma 2 9B

Type: SYSTEM

## From [[architectures-and-optimizations-for-speculative-de-micro05|architectures-and-optimizations-for-speculative-de-micro05]] (2026-06-09)
- A target model where drafting with Gemma 2 2B yields zero speedup within llama.cpp.
- Lacks direct distillation lineage or was trained on divergent datasets compared to Gemma 2 2B.

## From [[phantom-drive-autonomous-llm-deployment-architect-micro02|phantom-drive-autonomous-llm-deployment-architect-micro02]] (2026-06-10)
- A code/reasoning model developed by Google.
- Contains 9.24 billion parameters.
- Utilizes an interleaved structure of local and global attention mechanisms.
