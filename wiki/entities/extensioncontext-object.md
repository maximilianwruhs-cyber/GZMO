---
type: entity
title: ExtensionContext object
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# ExtensionContext object

Type: CONCEPT

## From [drive-research-building-pi-coding-agent-extensions](/entities/drive-research-building-pi-coding-agent-extensions.md) (2026-06-08)
- Provides interfaces for UI manipulation, state storage, and agent signaling.
- The session_start event handler receives this object.
- The session_shutdown event handler receives this object.
- The session_start event handler receives this object alongside the standard event payload object.
- Provides the interfaces necessary for UI manipulation, state storage, and agent signaling.
- Provides the ability to mount persistent widgets around the central coding editor.
- Includes the ctx.ui.setWidget() function.
- Allows dynamic updates or teardown of mount points.
