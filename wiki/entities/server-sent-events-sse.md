---
type: entity
title: Server-Sent Events (SSE)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Server-Sent Events (SSE)

Type: CONCEPT

## From [[drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01|drive-research-benchmarking-llamacpp-server-prefill-tokens-micro01]] (2026-06-09)
- Chunks emitted by llama-server when 'return_progress' is true and 'stream' is true.
- Contain prefill metrics before the first token is generated.
- Used to track progress during long sequence ingestion.
