---
type: entity
title: gVisor (runsc)
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---



# gVisor (runsc)

Type: SANDBOXING_TECHNOLOGY

## From [[ai-research-part6-micro04|ai-research-part6-micro04]] (2026-06-09)
- Used for kernel-level isolation in TaskForge
- Provides kernel-level isolation
- Used by TaskForge for agent execution environments

## From [[ai-research-part6-micro05|ai-research-part6-micro05]] (2026-06-09)
- Used for sandbox containers in OpenClaw-Contained.
- Intercepts system calls and isolates the agent in a user-space kernel.
- Provides complete encapsulation of all syscalls via a user-space kernel.
