---
type: entity
title: hidden gating mode
created: 2026-06-08
updated: 2026-06-08
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# hidden gating mode

Type: CONCEPT

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- Extracts and processes layer-specific hidden states from positive and negative prompts.
- Requires a complete forward pass of each reference prompt, resulting in a high hardware footprint.
- Maximizes cross-domain routing accuracy and generalization without requiring downstream fine-tuning.

## From [[drive-research-hidden-mode-technical-analysis-and-configurati|drive-research-hidden-mode-technical-analysis-and-configurati]] (2026-06-08)
- Utilizes layer-specific representations extracted from positive and negative reference prompts.
- Generates routing vectors that have maximal dot products with hidden states associated with positive reference prompts.
- Mathematically superior for direct deployment without downstream fine-tuning.

## From [[drive-research-hidden-mode-technical-analysis-and-configuration|drive-research-hidden-mode-technical-analysis-and-configuration]] (2026-06-08)
- Utilizes layer-specific representations extracted from positive and negative reference prompts.
- Outclasses random and cheap_embed modes.
- Generates routing vectors that have maximal dot products with hidden states associated with positive reference prompts while minimizing dot products with negative reference prompts.
