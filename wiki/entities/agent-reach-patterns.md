---
type: entity
title: "Agent-Reach Patterns"
created: "2026-06-20"
updated: "2026-06-20"
status: draft
sources: 1
tags:
  - research
  - agent-reach
  - compliance
---

# Agent-Reach Patterns

Agent-Reach integration under GZMO Sovereign Node compliance with a **Tier 2 network exception**.

## Network access

When `agent-reach` is listed in `[compliance].network_exceptions` (default), Agent-Reach CLIs and markers are **never blocked** by `compliance.rs`, even in `mode = "sovereign"`.

See [Network Exception Tier](/wiki/entities/network-exception-tier.md) for the full tier model.

## Workspace isolation (retained)

- Hermetic sandbox under `~/.agent-reach/` — isolates clones, tokens, and temp artifacts from the project tree
- ASH session material stays on the local host; no vault exfiltration

## Capabilities

- Ambient Session Hijacking (ASH) for authenticated platform state
- Dynamic fallback poly-routing across CLI backends
- Zero-wrapper CLI multiplexing into agent shell context

## Runtime enforcement

`gzmo-core/src/compliance.rs` — `shell_command_block_reason` skips Agent-Reach markers when the exception is active.
