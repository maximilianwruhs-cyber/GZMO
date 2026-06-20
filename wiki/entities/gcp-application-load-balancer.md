---
type: entity
title: GCP Application Load Balancer
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# GCP Application Load Balancer

Type: SYSTEM

## From [openclaw-part2](/entities/openclaw-part2.md) (2026-06-08)
- Can be reconfigured to use the IN_FLIGHT load balancing mode.
- Tracks the absolute number of concurrent active connections by setting backend trafficDuration to LONG.
- Safely spills traffic to other zones only when true concurrency thresholds are met.
