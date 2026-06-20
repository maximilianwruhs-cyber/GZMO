---
type: entity
title: Google Cloud Application Load Balancer
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Google Cloud Application Load Balancer

Type: SYSTEM

## From [openclaw-part2](/entities/openclaw-part2.md) (2026-06-08)
- Required for exposing the OpenClaw Gateway to external messaging APIs and remote developer clients.
- Conventional load balancing algorithms can pose a threat to agentic workloads.
- Supports the IN_FLIGHT balancing mode for long-lived connections.
- Utilizes IN_FLIGHT balancing to GKE pods.
- Integrates with Google Cloud Armor for robust Layer 7 defense.
- Routes traffic after inspection by Cloud Armor.
