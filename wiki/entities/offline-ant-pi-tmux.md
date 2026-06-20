---
type: entity
title: offline-ant/pi-tmux
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# offline-ant/pi-tmux

Type: TOOL

## From [drive-research-pi-coding-agent-ecosystem-tier-list](/entities/drive-research-pi-coding-agent-ecosystem-tier-list.md) (2026-06-08)
- Introduces file-based locking under /tmp/pi-semaphores/ using the pi-semaphore library.
- Maps lock file names directly to tmux pane IDs.
- Allows disparate agents operating on the same physical host to acquire lock files.
- A multiplexer session used by @romansix/pi-tmux.
- Enables asynchronous command execution inside persistent background windows.
- Can be managed via lock files in offline-ant/pi-tmux.

## From [drive-research-pi-coding-agent-local-deployment-customization](/entities/drive-research-pi-coding-agent-local-deployment-customization.md) (2026-06-08)
- An implementation of the pi-tmux extension.
