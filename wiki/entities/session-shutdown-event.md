---
type: entity
title: session_shutdown event
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# session_shutdown event

Type: CONCEPT

## From [[drive-research-building-pi-coding-agent-extensions|drive-research-building-pi-coding-agent-extensions]] (2026-06-08)
- Fires immediately before the runtime tears down the current extension instance or transitions away from the actively loaded session.
- Strictly designated for garbage collection and graceful termination.
- The Pi coding agent is designed to run persistently as a long-lived process in the terminal—handling multiple conversational turns, branching session trees, and dynamic context reloads.
- The session_start event is triggered whenever a session is initialized or when the runtime transitions between drastically different interaction states.
- The session_shutdown event fires immediately before the runtime tears down the current extension instance or transitions away from the actively loaded session.
