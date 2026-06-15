---
type: entity
title: Lazy Loading architecture
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Lazy Loading architecture

Type: CONCEPT

## From [[the-cognitive-architecture-of-openclaw-agents-micro02|the-cognitive-architecture-of-openclaw-agents-micro02]] (2026-06-09)
- Skills are not preemptively injected into the system prompt.
- The engine provides the model with a highly compressed metadata list outlining available capabilities.
- Dynamically reads the SKILL.md file from the disk and injects it into the active memory sequence.
