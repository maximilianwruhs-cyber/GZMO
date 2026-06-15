---
type: entity
title: Background Worker Tasks
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Background Worker Tasks

Type: SYSTEM

## From [[drive-research-rust-tui-architecture-tech-stack1-micro05|drive-research-rust-tui-architecture-tech-stack1-micro05]] (2026-06-09)
- Distinct tasks spawned to handle long-running operations.
- Execute operations like massive file reads.
- Must be wrapped in tokio::task::spawn_blocking for heavy synchronous CPU compute.

## From [[drive-research-rust-tui-architecture-tech-stack1-micro02|drive-research-rust-tui-architecture-tech-stack1-micro02]] (2026-06-10)
- Distinct tasks spawned to handle long-running operations.
- Executes operations like massive file reads or heavy CPU compute.
