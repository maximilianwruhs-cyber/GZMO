---
type: entity
title: Gateway Server
created: 2026-06-08
updated: 2026-06-08
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Gateway Server

Type: SYSTEM

## From [[openclaw-deep-research-part6|openclaw-deep-research-part6]] (2026-06-08)
- The central server process in OpenClaw.
- Coordinates between the LLM, the local machine, and messaging channels.
- Routes messages to specific sessions in OpenClaw.
- Manages concurrent requests.

## From [[openclaw-deep-research-part12|openclaw-deep-research-part12]] (2026-06-08)
- The 'heart' of OpenClaw.
- Routes messages to the correct session.
- Handles multiple overlapping requests.
