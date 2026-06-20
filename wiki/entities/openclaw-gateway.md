---
type: entity
title: OpenClaw Gateway
created: 2026-06-08
updated: 2026-06-09
sources: 4
tags: []
status: draft
gzmo_synthetic: true
---




# OpenClaw Gateway

Type: SYSTEM

## From [openclaw-part2](/entities/openclaw-part2.md) (2026-06-08)
- Serves as the central nervous system of the OpenClaw framework.
- Operates as a long-lived Node.js service.
- Exposes a strictly typed WebSocket API, operating by default on port 18789.
- Traffic originating from external messaging applications is first inspected by Cloud Armor.
- Manages sessions and relies on Filestore for state persistence.
- Securely negotiates outbound API connections to external LLMs such as Vertex AI or Anthropic.
- The central server process in OpenClaw that coordinates between the LLM, the local machine, and messaging channels.
- Configured to trigger a heartbeat every 30 minutes.
- Dispatches all visual and audio generation tasks asynchronously.
- An exceptionally sophisticated, open-source autonomous AI agent framework.
- Originally launched in November 2025 by Austrian developer Peter Steinberger under the moniker Clawdbot.
- Accumulated over 247,000 GitHub stars and 47,700 forks by early March 2026.
- Derived from its unrestricted ability to access the local file system, execute arbitrary shell commands, and interact dynamically with the broader internet.
- Stores highly sensitive credentials in plaintext Markdown and JSON files within the ~/.openclaw/ directory by default.
- Workloads must never be executed on host machines containing sensitive corporate data outside of the agent's explicit operational purview.
- A local-first, continuously running autonomous agent.
- Operates via a scheduled "heartbeat" (running 24/7 in the background).
- Uses a specific plugin architecture (SKILL.md files) and communicates via omnichannel messaging apps.

## From [openclaw-deep-research-part1-micro04](/entities/openclaw-deep-research-part1-micro04.md) (2026-06-09)
- Used to call config.get.
- Used to call config.apply.
- Used to call config.patch.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro06.md) (2026-06-09)
- Functions as the central "nervous system" of the agent.
- Abstracts communication protocols into a unified schema.
- Local daemon serving as the control plane.
- Manages connections to messaging platforms like WhatsApp, Telegram, Discord, and Slack.
- Operates a long-lived background process bound to a local WebSocket API.
- Implements a serialized Command Queue to prevent state corruption.

## From [the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08](/entities/the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro08.md) (2026-06-09)
- Acts as the central 'nervous system'.
- Abstracts communication protocols.
- Keeps the agent core platform-agnostic.
- Operates on port 18789 by default.
