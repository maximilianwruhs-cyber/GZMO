---
type: entity
title: Mixtral-style architectures
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Mixtral-style architectures

Type: CONCEPT

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- Assigns scores to each expert with a single matrix multiplication.
- Executes a dot product of a routing vector with the model's hidden state for each expert at each layer.
- Used as a reference for routing in transformer models.
- It is a target architecture that mergekit-moe natively supports outputting.
- It introduces distinct architectural constraints.
- Its architecture can be explicitly set in the YAML configuration.
