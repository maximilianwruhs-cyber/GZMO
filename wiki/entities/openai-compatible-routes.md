---
type: entity
title: OpenAI-compatible routes
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# OpenAI-compatible routes

Type: CONCEPT

## From [[drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01|drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01]] (2026-06-09)
- Includes routes like /v1/completions or /v1/chat/completions.
- Timing structures are not natively returned in standard JSON response headers.
- Measuring prefill speed requires monitoring streamed server-sent event chunk deltas or configuring Prometheus metrics trackers.
