---
type: entity
title: pi-local-rag
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# pi-local-rag

Type: TOOL

## From [[drive-research-pi-coding-agent-ecosystem-tier-list|drive-research-pi-coding-agent-ecosystem-tier-list]] (2026-06-08)
- Registers three native commands: rag_index, rag_query, and rag_status.
- Handles large text bases without exhausting the model's prompt buffer.
- Executes a hybrid search blending lexical retrieval with dense vector similarity.
- Used as a context pruning filter.
- Appends hidden metadata blocks with customType: "rag".
