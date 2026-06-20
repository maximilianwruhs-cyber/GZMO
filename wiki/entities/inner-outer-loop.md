---
type: entity
title: Inner-Outer Loop
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Inner-Outer Loop

Type: CONCEPT

## From [architectures-and-optimizations-for-speculative-de-micro02](/entities/architectures-and-optimizations-for-speculative-de-micro02.md) (2026-06-09)
- The first phase of the training process.
- Each agent is considered in isolation.
- Primary goal is to train the model-specific InnerLink module.
- Aims to generate stable, coherent, and semantically meaningful 'latent thoughts' over multiple timesteps.
- Functions as a 'warm-start' at the local model level.
- Ensures the internal semantic coherence of the agent before it interacts with other agents.
- A highly specialized learning algorithm developed by researchers.
- Addresses the critical problem of credit assignment in multi-agent structures.
- The training process is two-stage.
- The Inner Loop focuses on model-local warm-up.
- The Outer Loop involves system-wide gradient routing and BPTT.
