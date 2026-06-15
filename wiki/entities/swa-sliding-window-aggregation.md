---
type: entity
title: SWA (sliding-window aggregation)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# SWA (sliding-window aggregation)

Type: CONCEPT

## From [[ai-research-part1|ai-research-part1]] (2026-06-08)
- A simple way to reduce memory cost for cross-layer access.
- Retains only the most recent W=8 layer outputs plus the token embedding.
- Improves over baseline (1.764) but falls short of Full and Block AttnRes.
