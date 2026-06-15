---
type: entity
title: Gateway Session Hygiene layer
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Gateway Session Hygiene layer

Type: SYSTEM

## From [[drive-research-hermes-compression-and-bol-architecture|drive-research-hermes-compression-and-bol-architecture]] (2026-06-08)
- First line of defense in Hermes's dual-layer compression architecture.
- Asynchronous safety net located within the core routing process (gateway/run.py).
- Operates entirely pre-agent, intercepting incoming message payloads before the primary agent loop initializes prompt assembly.
- Trigger mechanism is hard-coded at 85% of the configured model's maximum context length.
- Utilizes a rapid, heuristic character-based estimation to gauge session history size.
