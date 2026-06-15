---
type: entity
title: Automatic Prefix Caching (APC)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Automatic Prefix Caching (APC)

Type: CONCEPT

## From [[drive-research-token-efficient-bol-processing-architecture|drive-research-token-efficient-bol-processing-architecture]] (2026-06-08)
- Implemented by vLLM.
- Allows new queries to skip the expensive prefill computation phase entirely if they share identical prompt prefixes with existing queries.
- Forces the Time to First Token (TTFT) to plummet for BoL extraction.
