---
type: entity
title: Chunked Prefill
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Chunked Prefill

Type: CONCEPT

## From [architectural-blueprints-for-sovereign-frankenmoe-part1](/entities/architectural-blueprints-for-sovereign-frankenmoe-part1.md) (2026-06-08)
- Balances prefill (compute-bound) and decode (memory-bound) phases.
- Splits long prompts into smaller chunks.
- Batches chunks with active decode requests.
- It works alongside Model Runner V2 (MRV2).
- It balances compute-bound prefill and memory-bound decode phases.
- It helps improve token-per-second throughput.
