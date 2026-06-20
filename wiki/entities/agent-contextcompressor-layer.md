---
type: entity
title: Agent ContextCompressor Layer
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Agent ContextCompressor Layer

Type: SYSTEM

## From [drive-research-hermes-compression-and-bol-architecture](/entities/drive-research-hermes-compression-and-bol-architecture.md) (2026-06-08)
- Primary context management engine in Hermes.
- Located within agent/context_compressor.py.
- Operates dynamically inside the agent's internal tool-execution loop.
- Trigger mechanism is a configurable percentage of the context window, defaulting to 50% (compression.threshold: 0.50).
- Accesses accurate, API-reported token consumption metrics.
