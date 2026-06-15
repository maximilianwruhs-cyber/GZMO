---
type: entity
title: Decoding Phase
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Decoding Phase

Type: CONCEPT

## From [[drive-research-hermes-anthropic-openrouter-cache-investigation|drive-research-hermes-anthropic-openrouter-cache-investigation]] (2026-06-08)
- The model operates iteratively.
- It calculates only the newly generated token and appends its state to the KV-Cache.
- The KV-Cache is 'append-only' during this phase.
