---
type: entity
title: PrfaaS-PD architecture
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# PrfaaS-PD architecture

Type: SYSTEM

## From [prfaas-cross-datacenter-llm-serving-via-selective-micro03](/entities/prfaas-cross-datacenter-llm-serving-via-selective-micro03.md) (2026-06-10)
- Comprises three subsystems: compute, network, and storage
- Leverages cross-datacenter KVCache to decouple prefill and decode
- Uses dedicated PrfaaS clusters for compute-intensive long-context prefill
