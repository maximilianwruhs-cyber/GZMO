---
type: entity
title: NUMA Balancing Script (disable-numa-balancing.sh)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# NUMA Balancing Script (disable-numa-balancing.sh)

Type: TOOL

## From [[drive-research-rust-ecs-cache-optimization-research|drive-research-rust-ecs-cache-optimization-research]] (2026-06-08)
- Temporarily disables the operating system's automated NUMA page migration.
- Prevents latency spikes and memory bus contention in multi-socket CPU topologies.
- Requires root/sudo privileges.
