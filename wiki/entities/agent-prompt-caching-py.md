---
type: entity
title: agent/prompt_caching.py
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# agent/prompt_caching.py

Type: CONCEPT

## From [drive-research-hermes-anthropic-openrouter-cache-investigation](/entities/drive-research-hermes-anthropic-openrouter-cache-investigation.md) (2026-06-08)
- It is a key focus in the investigation of agent memory.
- Hermes integrates a dedicated architecture for prompt caching.
- External API-level caching can be fragile.
- The primary prompt caching logic resides in this module.
- It is explicitly designed to utilize Anthropic's cache_control breakpoints.
