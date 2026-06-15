---
type: entity
title: Config
created: 2026-06-09
updated: 2026-06-10
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Config

Type: CONCEPT

## From [[openclaw-deep-research-part11-micro06|openclaw-deep-research-part11-micro06]] (2026-06-09)
- The main configuration structure for OpenClaw.
- Matches the existing OpenClaw JSON5 config schema.
- Contains fields for gateway, agents, channels, providers, and settings.
- Can be loaded from default or specific paths.
- Can be saved to a path.
- Has methods for getting default paths like `default_path()`, `state_dir()`, `credentials_dir()`, `sessions_dir()`, `agents_dir()`.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro03|resilient-rust-based-mcp-client-and-llm-orchestrat-micro03]] (2026-06-09)
- Portable Container Config.
- Contains image, cmd, host_config, and network_disabled fields.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro02|resilient-rust-based-mcp-client-and-llm-orchestrat-micro02]] (2026-06-10)
- Rust struct defining portable, host-agnostic aspects of a container.
