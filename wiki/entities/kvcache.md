---
type: entity
title: KVCache
created: 2026-06-10
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# KVCache

Type: CONCEPT

## From [prfaas-cross-datacenter-llm-serving-via-selective-micro02](/entities/prfaas-cross-datacenter-llm-serving-via-selective-micro02.md) (2026-06-10)
- A resource that must be transferred from prefill to decode instances.
- Size is reduced by hybrid-attention architectures, making cross-cluster transport more plausible.

## From [prfaas-cross-datacenter-llm-serving-via-selective-micro03](/entities/prfaas-cross-datacenter-llm-serving-via-selective-micro03.md) (2026-06-10)
- Transferred across clusters in cross-datacenter KVCache deployment
- Throughput is determined by inter-cluster bandwidth

## From [prfaas-cross-datacenter-llm-serving-via-selective-micro05](/entities/prfaas-cross-datacenter-llm-serving-via-selective-micro05.md) (2026-06-10)
- A system resource that can be transferred across datacenters.
- Size can be reduced via architectural techniques like MLA, sliding window attention, or linear attention.
- Can be compressed or reused via methods like H2O, KIVI, CacheGen, CacheBlend, and FusionRAG.
