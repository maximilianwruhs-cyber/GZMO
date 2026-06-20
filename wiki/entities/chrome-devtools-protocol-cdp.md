---
type: entity
title: Chrome DevTools Protocol (CDP)
created: 2026-06-08
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Chrome DevTools Protocol (CDP)

Type: SYSTEM

## From [drive-research-deep-dive-google-antigravity-architecture1](/entities/drive-research-deep-dive-google-antigravity-architecture1.md) (2026-06-08)
- The Browser Agent interfaces directly with the browser via a dedicated Antigravity Chrome extension.
- The MCP server detects the active Chrome instance on port 9222.
- The system searches for system-installed Chrome or Chromium binaries as a second-tier fallback.
- Lightpanda natively speaks this protocol.
- The Browser Agent utilizes CDP to actuate the browser interface.
- CDP provides low-level, high-fidelity access to the browser's core rendering engine and Document Object Model (DOM).
- The agent uses CDP endpoints to capture high-resolution snapshots of the DOM.

## From [openclaw-deep-research-part5-micro04](/entities/openclaw-deep-research-part5-micro04.md) (2026-06-09)
- Used by OpenClaw to automate browser tasks.

## From [openclaw-part1-micro04](/entities/openclaw-part1-micro04.md) (2026-06-10)
- An integration that allows the agent to control a browser instance.
- Enables navigating websites and bypassing captchas.
