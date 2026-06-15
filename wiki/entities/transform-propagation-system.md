---
type: entity
title: transform_propagation_system
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# transform_propagation_system

Type: SYSTEM

## From [[dynamics-of-the-unpredictable-micro07|dynamics-of-the-unpredictable-micro07]] (2026-06-09)
- Runs once every frame during rendering.
- Queries all entities possessing a Parent component.
- Writes the resulting matrix to the child's global transformation state.
