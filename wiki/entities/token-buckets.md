---
type: entity
title: Token Buckets
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---


# Token Buckets

Type: SYSTEM

## From [[drive-research-agentic-token-economy-blueprint-micro02|drive-research-agentic-token-economy-blueprint-micro02]] (2026-06-09)
- A strict rate limiter applied per identity tuple (user, project, model).
- Refills tokens continuously, allowing for specific burst capacity.
- Acts as the primary boundary for token consumption.
- Issues HTTP 429 errors if an agent exceeds its specific allocation.
