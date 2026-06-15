---
type: entity
title: cheap_embed gating mode
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# cheap_embed gating mode

Type: CONCEPT

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- Uses raw token embeddings directly from the input layer to build routing parameters.
- Applies a uniform matrix across all layers.
- Moderate-low routing quality due to lack of layer-specific context.

## From [[drive-research-hidden-mode-technical-analysis-and-configurati|drive-research-hidden-mode-technical-analysis-and-configurati]] (2026-06-08)
- Uses raw token embeddings directly from the input layer to build routing parameters.
- Lacks layer-specific routing context, leading to coarse routing decisions.
- Requires loading only the initial embedding layer.
