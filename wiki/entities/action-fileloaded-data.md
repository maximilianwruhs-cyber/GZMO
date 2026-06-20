---
type: entity
title: Action::FileLoaded(Data)
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Action::FileLoaded(Data)

Type: CONCEPT

## From [drive-research-rust-tui-architecture-tech-stack1-micro05](/entities/drive-research-rust-tui-architecture-tech-stack1-micro05.md) (2026-06-09)
- Sent back to the main channel upon completion of a background task.
- Carries data payload.
- Represents every possible state transition, background task trigger, and data payload.
- The lifeblood of the application.
- Used to send messages across the mpsc channel.
