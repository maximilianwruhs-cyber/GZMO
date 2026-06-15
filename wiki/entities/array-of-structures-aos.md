---
type: entity
title: Array of Structures (AoS)
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Array of Structures (AoS)

Type: CONCEPT

## From [[drive-research-cache-optimization-with-ai-chaos-theory|drive-research-cache-optimization-with-ai-chaos-theory]] (2026-06-08)
- A pattern where all telemetry and identifiers of a single agent are grouped into a single class or struct.
- Leads to severe spatial cache pollution and latency-inducing misses when scanning a single metric.
- Requires loading massive adjacent fields into the CPU's cache lines.

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- Traditional software architecture pattern.
- Groups all properties of a logical entity within a single class or struct.
- Introduces substantial performance penalties in high-frequency execution loops.
