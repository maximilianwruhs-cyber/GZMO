---
type: entity
title: Multi-stage Live Migration
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Multi-stage Live Migration

Type: CONCEPT

## From [drive-research-hermes-anthropic-openrouter-cache-investigation](/entities/drive-research-hermes-anthropic-openrouter-cache-investigation.md) (2026-06-08)
- Implemented by Llumnix.
- Overlaps the computation of new tokens with the copying of the historical KV-Cache.
- Minimizes downtime and prevents serving stalls.
