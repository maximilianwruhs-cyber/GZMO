---
type: entity
title: TopK function
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# TopK function

Type: CONCEPT

## From [architectural-blueprints-for-sovereign-frankenmoe-part1](/entities/architectural-blueprints-for-sovereign-frankenmoe-part1.md) (2026-06-08)
- Sets all elements outside the top-K indices to -\infty prior to applying the softmax operation.
- Yields a sparse routing vector.
- K is typically constrained to 1 or 2 in standard deployments.
