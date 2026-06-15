---
type: entity
title: PD cluster
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# PD cluster

Type: SYSTEM

## From [[prfaas-cross-datacenter-llm-serving-via-selective-micro04|prfaas-cross-datacenter-llm-serving-via-selective-micro04]] (2026-06-09)
- Performs PD-disaggregated serving.
- Can complete inference for a request end to end.
- Contains prefill nodes and decode nodes.

## From [[prfaas-cross-datacenter-llm-serving-via-selective-micro05|prfaas-cross-datacenter-llm-serving-via-selective-micro05]] (2026-06-10)
- A local cluster used for decoding and prefill.
- In the PrfaaS-PD configuration, it uses 64 H20 GPUs.
