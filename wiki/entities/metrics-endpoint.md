---
type: entity
title: /metrics Endpoint
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# /metrics Endpoint

Type: TOOL

## From [[drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01|drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01]] (2026-06-09)
- The native Prometheus exposition path hosted by llama-server.
- Maps internal engine metrics to standard observability fields.
- Aggregates data across all active instances when running with a --models-preset flag.
