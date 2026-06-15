---
type: source
title: drive-research-rust-tui-architecture-tech-stack1-micro02
created: 2026-06-10
updated: 2026-06-10
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-rust-tui-architecture-tech-stack1-micro02

Ingested source summary (2026-06-10).

## Entities
- [[src-config-rs|src/config.rs]] (PROJECT)
- [[main-ui-loop|Main UI Loop]] (SYSTEM)
- [[background-worker-tasks|Background Worker Tasks]] (SYSTEM)
- [[color-eyre|color-eyre]] (TOOL)
- [[webassembly|WebAssembly]] (CONCEPT)
- [[ratatui-themes|ratatui-themes]] (TOOL)
- [[zellij|Zellij]] (PROJECT)
- [[kdl|KDL]] (CONCEPT)
- [[src-main-rs|src/main.rs]] (PROJECT)
- [[nerd-fonts|Nerd Fonts]] (TOOL)
- [[serde|serde]] (TOOL)
- [[ratatui-themekit|ratatui-themekit]] (TOOL)
- [[src-action-rs|src/action.rs]] (PROJECT)
- [[src-tui-rs|src/tui.rs]] (PROJECT)
- [[tracing-appender|tracing_appender]] (TOOL)
- [[event-streamer-task|Event Streamer Task]] (SYSTEM)
- [[tokio-select|tokio::select!]] (TOOL)
- [[tui-logger|tui-logger]] (TOOL)
- [[tachyonfx|tachyonfx]] (TOOL)
- [[src-app-rs|src/app.rs]] (PROJECT)
- [[crossterm-event-eventstream|crossterm::event::EventStream]] (TOOL)
- [[tokio-time-interval|tokio::time::interval]] (TOOL)
- [[figment|figment]] (TOOL)
- [[clap|clap]] (TOOL)

## Relations
- Event Streamer Task → USES → crossterm::event::EventStream
- Main UI Loop → USES → tokio::select!
- Main UI Loop → USES → tokio::time::interval
- src/config.rs → USES → serde
- src/config.rs → USES → figment
- src/config.rs → USES → clap
- src/tui.rs → USES → crossterm
- Zellij → USES → WebAssembly
