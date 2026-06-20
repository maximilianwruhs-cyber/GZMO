---
type: entity
title: hermes-lcm plugin
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# hermes-lcm plugin

Type: TOOL

## From [drive-research-hermes-compression-and-bol-architecture](/entities/drive-research-hermes-compression-and-bol-architecture.md) (2026-06-08)
- An example of a Lossless Context Management (LCM) plugin.
- Utilizes an immutable SQLite message store to preserve every interaction verbatim.
- Constructs a depth-aware Directed Acyclic Graph (DAG) of the conversation.
- Injects specialized retrieval tools (lcm_grep, lcm_describe, lcm_expand) into the agent's payload.
