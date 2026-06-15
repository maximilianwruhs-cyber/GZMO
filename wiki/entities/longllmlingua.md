---
type: entity
title: LongLLMLingua
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# LongLLMLingua

Type: TOOL

## From [[drive-research-llmlingua-cpu-leistung-und-leistungstests|drive-research-llmlingua-cpu-leistung-und-leistungstests]] (2026-06-08)
- An extension of the original LLMLingua system.
- Designed to handle extremely long context scenarios.
- Performs query-aware compression and reorders context.
- Addresses the 'Lost in the Middle' phenomenon.

## From [[drive-research-token-efficient-bol-processing-architecture|drive-research-token-efficient-bol-processing-architecture]] (2026-06-08)
- Engineered specifically for long-context RAG systems.
- Utilizes query-aware compression.
- Measures the perplexity of document tokens conditionally against the user's specific query.
