---
type: entity
title: smol = true
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# smol = true

Type: CONCEPT

## From [[refactoring-gzmo-daemon-for-native-bun-high-perfor|refactoring-gzmo-daemon-for-native-bun-high-perfor]] (2026-06-08)
- A configuration setting in bunfig.toml.
- Forces the Bun heap to remain constrained.
- Aims to avoid Out-Of-Memory crashes.
- Incurs a slight CPU cost during GC.
