---
type: entity
title: exec
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# exec

Type: CONCEPT

## From [[openclaw-deep-research-part9-micro06|openclaw-deep-research-part9-micro06]] (2026-06-09)
- Source for SecretRef
- Reads from execution of commands (e.g., vault)

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06]] (2026-06-09)
- Core tool for verification.
- Grants ability to run shell commands on the Ubuntu host.
- Critical but high-risk capability, can grant root-level access.
- Often configured with "explicit consent" mode in production.
