---
type: entity
title: HashMap<AgentId, usize>
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# HashMap<AgentId, usize>

Type: TOOL

## From [drive-research-rust-ecs-cache-optimization-research](/entities/drive-research-rust-ecs-cache-optimization-research.md) (2026-06-08)
- Used by the hypervisor as a physical-to-logical translation table.
- Maps an agent's logical identity to its slot in parallel component vectors.
- Needs to be updated during swap-remove operations.
- A lightweight identifier for logical agents.
- Used in the Structure of Arrays (SoA) layout.
- Maintains logical association via a shared index.
