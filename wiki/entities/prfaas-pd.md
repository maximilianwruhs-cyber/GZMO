---
type: entity
title: PrfaaS-PD
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# PrfaaS-PD

Type: SYSTEM

## From [prfaas-cross-datacenter-llm-serving-via-selective-micro04](/entities/prfaas-cross-datacenter-llm-serving-via-selective-micro04.md) (2026-06-09)
- A system comprising PrfaaS and local PD clusters.
- Increases deployment scalability and lowers cost.
- Functions as a selective extension to conventional intra-cluster PD disaggregation.

## From [prfaas-cross-datacenter-llm-serving-via-selective-micro05](/entities/prfaas-cross-datacenter-llm-serving-via-selective-micro05.md) (2026-06-10)
- A cluster used for offloading compute-intensive long-context prefill.
- Currently compute-bound with ample bandwidth headroom.
- A disaggregation architecture that augments system serving throughput at low cost.
- Uses heterogeneous PrfaaS clusters connected via commodity Ethernet.
- Achieves 54% higher throughput and 64% lower P90 TTFT over a homogeneous PD-only baseline.
