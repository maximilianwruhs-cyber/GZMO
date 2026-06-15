---
type: entity
title: openclaw_core
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# openclaw_core

Type: PROJECT

## From [[openclaw-deep-research-part11-micro06|openclaw-deep-research-part11-micro06]] (2026-06-09)
- Runs work in the background through tasks, scheduled jobs, event hooks, and standing instructions.
- Uses a Gateway's built-in scheduler for precise timing.
- Has a background task ledger that tracks all detached work.
- Manages durable multi-step flows with managed and mirrored sync modes.
- Grants the agent permanent operating authority for defined programs.
- Hooks are event-driven scripts triggered by agent lifecycle events.
- Heartbeat is a periodic main-session turn.
- Configuration is managed via JSON5 schema.
- Has a core library `openclaw_core`.
- A Rust library.
- Contains configuration modules like `openclaw_core::config`.
- Has dependencies like `tokio`, `serde`, `reqwest`.

## From [[the-cognitive-architecture-of-openclaw-agents-micro04|the-cognitive-architecture-of-openclaw-agents-micro04]] (2026-06-09)
- Part of the Cargo workspace.
- Contains foundational traits, schemas, and LLM API gateway abstractions.
