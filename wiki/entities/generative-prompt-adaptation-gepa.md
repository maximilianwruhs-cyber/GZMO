---
type: entity
title: Generative Prompt Adaptation (GEPA)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Generative Prompt Adaptation (GEPA)

Type: ALGORITHM

## From [[engineering-ignorance-eliminating-affirmative-bia|engineering-ignorance-eliminating-affirmative-bia]] (2026-06-08)
- Treats prompts as parameters that can be iteratively refined using task-level feedback.
- Iteratively selects candidate prompts under a fixed optimization constraint.
- Applies reflective updates using execution traces.
- Operates entirely in discrete prompt space.
- Supports multi-objective selection (balancing accuracy and latency).
- Exceptionally well-suited for tuning constrained extraction prompts for SLMs.
