---
type: entity
title: HEARTBEAT.md
created: 2026-06-08
updated: 2026-06-10
sources: 11
tags: []
status: draft
gzmo_synthetic: true
---











# HEARTBEAT.md

Type: CONCEPT

## From [[openclaw-part2|openclaw-part2]] (2026-06-08)
- A directive that acts as the chronological pulse, waking GZMO on a schedule.
- Configures the gateway to trigger a heartbeat every 30 minutes.
- Instructs GZMO to perform background tasks, execute lightweight deterministic scripts, and monitor logs.

## From [[openclaw-autonomous-ai-agents-in-financial-operat|openclaw-autonomous-ai-agents-in-financial-operat]] (2026-06-08)
- Core scheduling file for OpenClaw.
- Proactively wakes the agent without human prompting.
- Synchronized with European trading day.
- Configured to wake the agent for analysis.
- Used for morning gap-up/down analysis and intraday momentum tracking.

## From [[openclaw-deep-research-part10-micro04|openclaw-deep-research-part10-micro04]] (2026-06-09)
- The autonomy layer for an OpenClaw agent.
- Defines scheduled tasks in plain English.
- The heartbeat daemon runs every 30 minutes by default.

## From [[the-cognitive-architecture-of-openclaw-agents-micro02|the-cognitive-architecture-of-openclaw-agents-micro02]] (2026-06-09)
- Functions as the chronological pulse of the artificial intelligence state.
- Allows the agent to break free from the constraints of prompt-driven architectures.
- Instructs the system to perform deterministic 'cheap checks' before invoking the primary inference engine.

## From [[the-cognitive-architecture-of-openclaw-agents-micro03|the-cognitive-architecture-of-openclaw-agents-micro03]] (2026-06-09)
- Drives proactive autonomy.

## From [[the-cognitive-architecture-of-openclaw-agents-micro04|the-cognitive-architecture-of-openclaw-agents-micro04]] (2026-06-09)
- Associated with the openclaw-daemon crate.
- Focuses on temporal autonomy via deterministic 'cheap checks'.

## From [[the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02|the-openclaw-architecture-and-tri-circuit-autonomo-part1-micro02]] (2026-06-09)
- A checklist located in the workspace that the agent parses upon waking.
- Can be configured for continuous Open-Source Intelligence (OSINT) monitoring.

## From [[openclaw-deep-research-part1-micro06|openclaw-deep-research-part1-micro06]] (2026-06-10)
- A file in the agent workspace used for periodic checks.
- Can be updated via agent prompts to add tasks like daily calendar checks.
- Should not contain secrets like API keys or private tokens.

## From [[openclaw-part1-micro02|openclaw-part1-micro02]] (2026-06-10)
- Used as a clock/taktgeber for the pi-mono runtime

## From [[openclaw-part1-micro05|openclaw-part1-micro05]] (2026-06-10)
- Acts as a deterministic gatekeeper and proactive clock.
- Wakes the system at configurable intervals (e.g., every 30 minutes).
- Uses 'Cheap Checks First' to minimize LLM invocation costs.

## From [[prompt-agent-engineering-part4-micro05|prompt-agent-engineering-part4-micro05]] (2026-06-10)
- A file for proactive state and triggers
