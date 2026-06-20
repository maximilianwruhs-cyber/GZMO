---
type: entity
title: ratatui::restore()
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# ratatui::restore()

Type: TOOL

## From [drive-research-rust-tui-architecture-tech-stack1-micro05](/entities/drive-research-rust-tui-architecture-tech-stack1-micro05.md) (2026-06-09)
- Disables raw mode and issues the LeaveAlternateScreen ANSI sequence.
- Restores the terminal to a normal, sane state.
- Must be called within a custom panic hook.
