---
type: entity
title: ExtensionAPI interface
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# ExtensionAPI interface

Type: CONCEPT

## From [drive-research-building-pi-coding-agent-extensions](/entities/drive-research-building-pi-coding-agent-extensions.md) (2026-06-08)
- Offers programmatic control over the agent's behavior.
- Developers can intercept agent events, define tool schemas, implement TUIs, and manage session states.
- The pi object implements this interface.
- Exposes an intricate event-driven architecture, primarily anchored by two critical lifecycle hooks: session_start and session_shutdown.

## From [high-performance-typescript-execution-and-architec-part1-micro05](/entities/high-performance-typescript-execution-and-architec-part1-micro05.md) (2026-06-09)
- Implemented by the 'pi' object.
- Provides interfaces for UI manipulation, state storage, and agent signaling.
- Exposes lifecycle hooks like session_start and session_shutdown.
- Allows developers to intercept agent events, define complex tool schemas, implement interactive TUIs, and manage persistent session states.
- Implemented by the 'pi' object passed to the default factory function.
- Exposes an intricate event-driven architecture anchored by lifecycle hooks.
