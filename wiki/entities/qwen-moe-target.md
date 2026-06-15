---
type: entity
title: Qwen MoE Target
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Qwen MoE Target

Type: CONCEPT

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- This architecture incorporates a 'shared expert' that remains active for all tokens.
- The merge configuration must specify exactly one shared expert.
- All input models must share the same base architecture (e.g., Llama, Mistral, or Qwen2).
