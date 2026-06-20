---
type: entity
title: global KVCache manager
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# global KVCache manager

Type: SYSTEM

## From [prfaas-cross-datacenter-llm-serving-via-selective-micro04](/entities/prfaas-cross-datacenter-llm-serving-via-selective-micro04.md) (2026-06-09)
- Maintains KVCache metadata across all clusters.
- Computes prefix-match information for every cluster.
- Performs cache rebalancing to mitigate hotspots.
- Stores recurrent states of linear attention or SWA layers.
- Can be transferred across clusters.
- Managed by separate groups with aligned block sizes in hybrid models.
