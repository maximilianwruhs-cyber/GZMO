---
type: entity
title: PrfaaS cluster
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# PrfaaS cluster

Type: SYSTEM

## From [[prfaas-cross-datacenter-llm-serving-via-selective-micro04|prfaas-cross-datacenter-llm-serving-via-selective-micro04]] (2026-06-09)
- Provides selective remote prefill capacity.
- Functions as a stateless KVCache producer.
- Dedicated to long-context prefill requests.

## From [[prfaas-cross-datacenter-llm-serving-via-selective-micro03|prfaas-cross-datacenter-llm-serving-via-selective-micro03]] (2026-06-10)
- Performs compute-intensive long-context prefill
- Uses cost-effective, high-throughput accelerators
- Streams resulting KVCache to local PD clusters via commodity Ethernet
