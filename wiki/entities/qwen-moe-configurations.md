---
type: entity
title: Qwen MoE configurations
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Qwen MoE configurations

Type: CONCEPT

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- Support a shared expert block.
- Shared expert is activated for all incoming tokens, running in parallel with routed experts.
- Applies a downweighting scaling parameter \alpha to the shared expert's output.
- An example of an upcycled model architecture that supports a shared expert block.
- A shared expert is activated for all incoming tokens, running in parallel with routed experts.
- It is a target architecture that mergekit-moe natively supports outputting.
- It introduces distinct architectural constraints.
- Its architecture can be explicitly set in the YAML configuration.
