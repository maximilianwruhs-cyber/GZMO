---
type: entity
title: Double-buffering
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Double-buffering

Type: CONCEPT

## From [[drive-research-rust-tui-architecture-tech-stack1-micro01|drive-research-rust-tui-architecture-tech-stack1-micro01]] (2026-06-09)
- Implemented by ratatui to prevent screen flickering and visual tearing.
- Maintains two in-memory grids: current frame buffer and previous frame buffer.
- Computes a strict mathematical diff between the new buffer and the previous frame's buffer.

## From [[drive-research-rust-tui-architecture-tech-stack1-micro04|drive-research-rust-tui-architecture-tech-stack1-micro04]] (2026-06-09)
- Sophisticated system implemented by ratatui to prevent screen flickering and visual tearing.
- Maintains two in-memory grids: current frame buffer and previous frame buffer.
- Computes a diff between new and previous buffer to minimize ANSI escape sequences.
