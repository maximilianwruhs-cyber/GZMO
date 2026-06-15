---
type: entity
title: DeepSeek MoE Target
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# DeepSeek MoE Target

Type: MODEL

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- It is a target architecture that mergekit-moe natively supports outputting.
- It introduces distinct architectural constraints.
- Its architecture can be explicitly set in the YAML configuration.
- This architecture uses fine-grained expert segmentation.
- It divides a standard FFN layer into multiple parallel, narrow sub-experts.
- This increases the total expert count and routing flexibility without scaling up the global parameter footprint.
