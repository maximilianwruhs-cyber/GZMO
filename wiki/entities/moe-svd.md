---
type: entity
title: MoE-SVD
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# MoE-SVD

Type: CONCEPT

## From [[architectural-blueprints-for-sovereign-frankenmoe-part1|architectural-blueprints-for-sovereign-frankenmoe-part1]] (2026-06-08)
- Imposes a shared input projection matrix across experts.
- Prunes the output projection matrix.
- Exploits the architectural assumption that expert redundancy is concentrated in the input mapping to optimize memory usage.
- Exploits the architectural assumption that expert redundancy is concentrated in the input mapping.

## From [[drive-research-frankenmoe-merging-ai-models|drive-research-frankenmoe-merging-ai-models]] (2026-06-08)
- Imposes a shared input projection matrix across experts.
- Prunes the output projection matrix.
- A compression technique for MoE models.
