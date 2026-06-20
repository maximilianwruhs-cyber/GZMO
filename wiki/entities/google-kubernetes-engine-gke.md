---
type: entity
title: Google Kubernetes Engine (GKE)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Google Kubernetes Engine (GKE)

Type: SYSTEM

## From [openclaw-part2](/entities/openclaw-part2.md) (2026-06-08)
- Advanced orchestration used by Google Cloud Platform.
- The undisputed industry standard for deploying OpenClaw production workloads.
- Pods are ephemeral by design.
- Traditional stateless web hosting paradigm that is incompatible with OpenClaw's stateful nature.
- Pods are ephemeral, which challenges OpenClaw's reliance on continuous local file mutations.
- Security contexts can be enforced for non-root users, dropped Linux capabilities, and read-only root filesystems.
- Provides necessary primitives for declarative deployment, environment promotion, and strict ownership boundaries.
- A hardened, repeatable environment where OpenClaw can operate continuously as a persistent backend service.
