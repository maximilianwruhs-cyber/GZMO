---
type: entity
title: cross-test state mutations
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# cross-test state mutations

Type: CONCEPT

## From [[drive-research-bun-typescript-performance-tips-micro03|drive-research-bun-typescript-performance-tips-micro03]] (2026-06-09)
- Setting randomize = true can expose hidden cross-test state mutations.
- Setting randomize = true with a specific integer seed forces the runner to execute the test suite in a chaotic, reproducible order, exposing hidden cross-test state mutations.
