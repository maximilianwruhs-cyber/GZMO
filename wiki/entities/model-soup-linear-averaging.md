---
type: entity
title: Model Soup (Linear Averaging)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Model Soup (Linear Averaging)

Type: ALGORITHM

## From [architectural-blueprints-for-sovereign-frankenmoe-part1](/entities/architectural-blueprints-for-sovereign-frankenmoe-part1.md) (2026-06-08)
- Linearly interpolates weights: W_{\text{merged}} = \sum_{i=1}^N \alpha_i W_i.
- Occasionally optimized greedily based on validation accuracy.
- Primary use case is baseline merging of models trained on identical datasets with different hyperparameters.
- It is a merge method.
- It involves simple weighted parameter averaging.
- It is scalable to any number of models.
