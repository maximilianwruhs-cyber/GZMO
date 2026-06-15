---
type: entity
title: LogViewer
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# LogViewer

Type: SYSTEM

## From [[drive-research-rust-tui-architecture-tech-stack1-micro05|drive-research-rust-tui-architecture-tech-stack1-micro05]] (2026-06-09)
- Can hold its own history buffers, cursor positions, and scrolling logic.
- Handles Action::ScrollDown messages directly.
- Does not clutter or alert the global application model.
