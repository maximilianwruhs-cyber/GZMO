---
type: entity
title: Run:ai
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Run:ai

Type: TOOL

## From [aether-grid-micro01](/entities/aether-grid-micro01.md) (2026-06-09)
- Used for scheduling on GH200-Cluster.
- Must be configured to prioritize vector ingestion.
- Can cause Kernel Panic if NVLink memory status is blocked.

## From [aether-grid-micro04](/entities/aether-grid-micro04.md) (2026-06-09)
- Controls GPU scheduling in the Core K8s cluster.

## From [aether-grid-micro03](/entities/aether-grid-micro03.md) (2026-06-09)
- Used for load balancing.
- Activated for GPU scheduling in the Core cluster in Phase 3.
