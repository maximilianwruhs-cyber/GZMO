---
type: entity
title: notify
created: 2026-06-08
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# notify

Type: TOOL

## From [[drive-research-pi-coding-agent-ecosystem-tier-list|drive-research-pi-coding-agent-ecosystem-tier-list]] (2026-06-08)
- Monitors agent state changes and fires system notifications.
- Notifications include terminal bell sounds, OSC 777 escape sequences, or desktop alerts.
- Fired when background compilation tasks complete and the agent requires user feedback.

## From [[drive-research-the-pi-coding-agent-s-architectural-paradigm-revol|drive-research-the-pi-coding-agent-s-architectural-paradigm-revol]] (2026-06-08)
- System alerts (OSC 777 or audio) when background tasks finish.
- B-Tier resource.
- Employs terminal bell audio, desktop notices, or Telegram webhooks to ping you when a long-running task completes.

## From [[the-cognitive-architecture-of-openclaw-agents-micro04|the-cognitive-architecture-of-openclaw-agents-micro04]] (2026-06-09)
- Used for file watching.
- Implemented in openclaw-identity for SOUL.md hot-reloading.
