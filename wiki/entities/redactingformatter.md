---
type: entity
title: RedactingFormatter
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# RedactingFormatter

Type: TOOL

## From [drive-research-hermes-compression-and-bol-architecture](/entities/drive-research-hermes-compression-and-bol-architecture.md) (2026-06-08)
- specialized formatter used by log handlers
- ensures that authentication tokens, API keys, and environmentally injected secrets are dynamically scrubbed before the data is committed to disk
