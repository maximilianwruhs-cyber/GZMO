---
type: entity
title: Immediate-mode rendering paradigm
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Immediate-mode rendering paradigm

Type: CONCEPT

## From [drive-research-rust-tui-architecture-tech-stack1-micro01](/entities/drive-research-rust-tui-architecture-tech-stack1-micro01.md) (2026-06-09)
- The application does not retain a persistent, stateful tree of widget objects in memory across frames.
- The entire user interface is procedurally re-declared and rebuilt during every single frame tick based on the current underlying application state.

## From [drive-research-rust-tui-architecture-tech-stack1-micro04](/entities/drive-research-rust-tui-architecture-tech-stack1-micro04.md) (2026-06-09)
- Application does not retain a persistent, stateful tree of widget objects across frames.
- Entire user interface is procedurally re-declared and rebuilt during every frame tick.
- Paired with double-buffering in ratatui.
