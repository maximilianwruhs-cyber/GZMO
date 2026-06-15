---
type: entity
title: src/midi.rs
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# src/midi.rs

Type: CONCEPT

## From [[prompt-agent-engineering-part2-micro05|prompt-agent-engineering-part2-micro05]] (2026-06-09)
- High-fidelity output from the sonification engine
- Separate channels for different metrics
- Real-time priority for dispatch
- Manages RT thread (SCHED_FIFO), midir dispatch, and rtrb Consumer
