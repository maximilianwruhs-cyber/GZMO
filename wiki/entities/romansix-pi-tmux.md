---
type: entity
title: '@romansix/pi-tmux'
created: 2026-06-08
updated: 2026-06-08
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# @romansix/pi-tmux

Type: TOOL

## From [[drive-research-pi-coding-agent-ecosystem-tier-list|drive-research-pi-coding-agent-ecosystem-tier-list]] (2026-06-08)
- Assigns a dedicated tmux multiplexer session to each Git repository root.
- Registers the tmux tool, enabling the agent to execute commands asynchronously.
- Uses silence detection algorithms to monitor background shell activity.

## From [[drive-research-pi-coding-agent-local-deployment-customization|drive-research-pi-coding-agent-local-deployment-customization]] (2026-06-08)
- Manages background tasks by spawning commands in new tmux windows.
- Implements a silence detection algorithm.
- A terminal multiplexer that can be used to run Pi.
- Requires configuration to use extended keys formatted as CSI-u sequences.
- A tmux controller extension that can spawn child worker sessions.

## From [[drive-research-the-pi-coding-agent-s-architectural-paradigm-revol|drive-research-the-pi-coding-agent-s-architectural-paradigm-revol]] (2026-06-08)
- Persistent background runs with output peeking and silence monitors.
- S-Tier resource.
- Provides the ultimate terminal multitasking harness.
