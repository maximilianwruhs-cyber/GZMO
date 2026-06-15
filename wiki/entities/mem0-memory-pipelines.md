---
type: entity
title: Mem0 memory pipelines
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Mem0 memory pipelines

Type: SYSTEM

## From [[drive-research-hermes-compression-and-bol-architecture|drive-research-hermes-compression-and-bol-architecture]] (2026-06-08)
- First-class memory provider plugin for Hermes (plugins/memory/).
- Orchestrates a continuous, asynchronous pipeline for persistent memory integration.
- Utilizes an independent LLM call to identify new preferences, constraints, decisions, and entity relationships.
- system is heavily augmented by these decoupled, persistent pipelines
- tool is invoked with an inference bypass parameter (infer=False)
- forces the system to save the exact string verbatim to the persistent disk, bypassing the semantic extraction layer entirely
