---
type: entity
title: Qwen MoE
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# Qwen MoE

Type: CONCEPT

## From [drive-research-hidden-mode-technical-analysis-and-configurati](/entities/drive-research-hidden-mode-technical-analysis-and-configurati.md) (2026-06-08)
- Configurations that support a shared expert block.
- A shared expert is activated for all incoming tokens, running in parallel with the routed experts.
- Requires a downweighting scaling parameter (alpha) to prevent overcooking.

## From [drive-research-hidden-mode-technical-analysis-and-configuration](/entities/drive-research-hidden-mode-technical-analysis-and-configuration.md) (2026-06-08)
- Configurations that support a shared expert block.
- A shared expert is activated for all incoming tokens, running in parallel with the routed experts.
- Requires a downweighting scaling parameter \alpha to prevent the shared block from dominating the output.

## From [drive-research-mergekit-moe-model-creation-guide](/entities/drive-research-mergekit-moe-model-creation-guide.md) (2026-06-08)
- A target architecture for MoE output.
- Incorporates a 'shared expert' that remains active for all tokens alongside the routed experts.
- A target architecture supported by mergekit-moe.
- Incorporates a 'shared expert' that remains active for all tokens alongside routed experts.

## From [architectures-and-optimizations-for-speculative-de-micro06](/entities/architectures-and-optimizations-for-speculative-de-micro06.md) (2026-06-09)
- Variants are examples of Sparse Mixture of Experts (MoE) models.
