---
type: entity
title: EventHandler
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# EventHandler

Type: SYSTEM

## From [[drive-research-rust-tui-architecture-tech-stack1-micro05|drive-research-rust-tui-architecture-tech-stack1-micro05]] (2026-06-09)
- New instance created with an action_tx.
- Has a next() method to get events.
- Responsible for parsing asynchronous EventStream inputs.
