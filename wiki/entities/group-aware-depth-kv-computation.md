---
type: entity
title: Group-aware depth KV computation
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Group-aware depth KV computation

Type: CONCEPT

## From [[ai-research-part5|ai-research-part5]] (2026-06-08)
- Leverages the mapping Tq = G*Tkv where adjacent query rows share the same base-time index.
- Reduces the required depth span by reusing depth KV blocks.
