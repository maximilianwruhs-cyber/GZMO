---
type: entity
title: Gateway
created: 2026-06-09
updated: 2026-06-10
sources: 7
tags: []
status: draft
gzmo_synthetic: true
---







# Gateway

Type: SYSTEM

## From [[openclaw-deep-research-part11-micro04|openclaw-deep-research-part11-micro04]] (2026-06-09)
- The single process through which everything in OpenClaw flows.
- Described as the 'single source of truth' for sessions, routing, and channel connections.
- Acts as the nervous system of the whole system.
- Typically run as a long-lived background process.
- Handles routing, connectivity, authentication, and session management.
- An orchestration layer in front of the model.
- A controlled process that handles routing, queuing, and state management.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro01|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro01]] (2026-06-09)
- The central 'nervous system' of OpenClaw.
- Runs as a long-lived background daemon.
- Managed by supervisors like systemd or LaunchAgent.
- Binds a WebSocket API to ws://127.0.0.1:18789 by default.
- Exposes a typed interface for requests, responses, and server-push events.
- Plays a critical role in data ingestion and normalization for fact-checking agents.
- Utilizes dedicated Channel Adapters.

## From [[openclaw-deep-research-part7-micro06|openclaw-deep-research-part7-micro06]] (2026-06-10)
- An always-on process that acts as the control plane
- Manages message ingress/egress, sessions, and tool execution
- Runs on a host machine like a Mac mini or VPS

## From [[openclaw-deep-research-part7-micro07|openclaw-deep-research-part7-micro07]] (2026-06-10)
- Can run multiple channels and agents
- Plugins run in-process with the Gateway
- Requires isolation of config path, state dir, workspace, and port

## From [[openclaw-deep-research-part9-micro01|openclaw-deep-research-part9-micro01]] (2026-06-10)
- A single long-lived Node.js daemon that owns all state and connections.
- Exposes a typed WebSocket API.
- Acts as the central nervous system of the architecture.

## From [[openclaw-deep-research-part9-micro02|openclaw-deep-research-part9-micro02]] (2026-06-10)
- Acts as a single-writer system for each session
- Fires heartbeats every N minutes
- Discovers hooks from three directories: workspace, managed, and bundled

## From [[openclaw-deep-research-part9-micro05|openclaw-deep-research-part9-micro05]] (2026-06-10)
- Exposes config.schema.lookup to fetch path-scoped schema nodes
- Refuses to start if configuration fails strict validation
- Watches the config file and applies changes automatically
