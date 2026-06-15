---
type: entity
title: Non-P2P PCIe Topology Constraints
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Non-P2P PCIe Topology Constraints

Type: CONCEPT

## From [[optimizing-nvidia-blackwell-sm120-part3-micro05|optimizing-nvidia-blackwell-sm120-part3-micro05]] (2026-06-09)
- Major impact on multi-GPU configuration stability.
- Occurs when GPUs are on asymmetric PCIe lanes.
- Disables direct Peer-to-Peer (P2P) memory access.
