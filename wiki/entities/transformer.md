---
type: entity
title: Transformer
created: 2026-06-08
updated: 2026-06-09
sources: 5
tags: []
status: draft
gzmo_synthetic: true
---





# Transformer

Type: SYSTEM

## From [[ai-research-part1|ai-research-part1]] (2026-06-08)
- Improved upon RNNs by replacing recurrence with attention.
- Each self-attention or MLP is treated as an individual layer in Transformer models.

## From [[the-sovereign-software-factory-blueprint|the-sovereign-software-factory-blueprint]] (2026-06-08)
- Uses 'inner product' math for Attention calculation.

## From [[drive-research-frankenmoe-blueprint-analysis|drive-research-frankenmoe-blueprint-analysis]] (2026-06-08)
- The architectural basis of the FrankenMoE paradigm lies in decoupling functional representations within its block.
- Parameters are split between self-attention layers and multilayer perceptrons (FFNs).

## From [[drive-research-hidden-mode-technical-analysis-and-configurati|drive-research-hidden-mode-technical-analysis-and-configurati]] (2026-06-08)
- Parameters are partitioned between self-attention layers and feed-forward networks (FFN).
- Self-attention layers govern context, sequence topology, and linguistic dependencies.
- FFN layers act as key-value databases storing factual, structural, and domain-specific knowledge.

## From [[architectures-and-optimizations-for-speculative-de-micro04|architectures-and-optimizations-for-speculative-de-micro04]] (2026-06-09)
- Neural network architecture.
- Processes sequences of tokens with exceptional arithmetic intensity during prefill/prompt-processing.
