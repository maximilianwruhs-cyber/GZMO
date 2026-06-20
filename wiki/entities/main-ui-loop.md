---
type: entity
title: Main UI Loop
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Main UI Loop

Type: SYSTEM

## From [drive-research-rust-tui-architecture-tech-stack1-micro05](/entities/drive-research-rust-tui-architecture-tech-stack1-micro05.md) (2026-06-09)
- Orchestrates everything using the tokio::select! macro.
- Concurrently awaits multiple asynchronous branches.
- Guarantees a consistent UI framerate.

## From [drive-research-rust-tui-architecture-tech-stack1-micro02](/entities/drive-research-rust-tui-architecture-tech-stack1-micro02.md) (2026-06-10)
- The main thread orchestrates everything using the tokio::select! macro.
- Multiplexes crossterm events, background Action payloads, and a tokio::time::interval.
