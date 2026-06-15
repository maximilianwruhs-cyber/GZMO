---
type: source
title: drive-research-rust-tui-architecture-tech-stack1-micro05
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-rust-tui-architecture-tech-stack1-micro05

Ingested source summary (2026-06-09).

## Entities
- [[color-eyre-config-hookbuilder|color_eyre::config::HookBuilder]] (TOOL)
- [[tracing-appender-non-blocking|tracing_appender::non_blocking]] (TOOL)
- [[flexbox|Flexbox]] (CONCEPT)
- [[rust-tracing-ecosystem|Rust tracing ecosystem]] (TOOL)
- [[alternate-screen|Alternate Screen]] (CONCEPT)
- [[tui-logger|tui-logger]] (TOOL)
- [[src-main-rs|src/main.rs]] (SYSTEM)
- [[asynchronous-event-loop|Asynchronous Event Loop]] (CONCEPT)
- [[mouseeventkind-down-mousebutton-left|MouseEventKind::Down(MouseButton::Left)]] (CONCEPT)
- [[ratatui-themekit|ratatui-themekit]] (TOOL)
- [[background-worker-tasks|Background Worker Tasks]] (SYSTEM)
- [[eventhandler|EventHandler]] (SYSTEM)
- [[src-config-rs|src/config.rs]] (SYSTEM)
- [[std-panic-set-hook|std::panic::set_hook]] (TOOL)
- [[mpsc-unbounded-channel|mpsc unbounded channel]] (SYSTEM)
- [[tachyonfx|tachyonfx]] (TOOL)
- [[crossterm-event-eventstream|crossterm::event::EventStream]] (TOOL)
- [[println|println!]] (TOOL)
- [[clear-widget|Clear widget]] (TOOL)
- [[src-components-mod-rs|src/components/mod.rs]] (SYSTEM)
- [[log-info|log::info!]] (TOOL)
- [[crossterm-backend|crossterm backend]] (TOOL)
- [[app-run|App::run()]] (SYSTEM)
- [[focusable-trait|Focusable trait]] (CONCEPT)
- [[event-enum|Event enum]] (CONCEPT)
- [[constraint-variants|Constraint variants]] (CONCEPT)
- [[src-components-log-viewer-rs|src/components/log_viewer.rs]] (SYSTEM)
- [[component-trait|Component trait]] (CONCEPT)
- [[nord|Nord]] (CONCEPT)
- [[src-tui-rs|src/tui.rs]] (SYSTEM)
- [[position|Position]] (CONCEPT)
- [[color-eyre|color-eyre]] (TOOL)
- [[dracula|Dracula]] (CONCEPT)
- [[style-default-fg-color-rgb-x-y-z|Style::default().fg(Color::Rgb(x,y,z))]] (TOOL)
- [[event-streamer-task|Event Streamer Task]] (SYSTEM)
- [[ratatui-restore|ratatui::restore()]] (TOOL)
- [[main-ui-loop|Main UI Loop]] (SYSTEM)
- [[truecolor|Truecolor]] (CONCEPT)
- [[drive-research-rust-tui-architecture-tech-stack1|drive-research-rust-tui-architecture-tech-stack1]] (PROJECT)
- [[logviewer|LogViewer]] (SYSTEM)
- [[catppuccin|Catppuccin]] (CONCEPT)
- [[tokyo-night|Tokyo Night]] (CONCEPT)
- [[action-fileloaded-data|Action::FileLoaded(Data)]] (CONCEPT)
- [[ratatui-interact|ratatui-interact]] (TOOL)
- [[dbg|dbg!]] (TOOL)
- [[raw-mode|Raw Mode]] (CONCEPT)
- [[rect|Rect]] (CONCEPT)
- [[src-app-rs|src/app.rs]] (SYSTEM)
- [[src-action-rs|src/action.rs]] (SYSTEM)
- [[tracing-logger|tracing logger]] (TOOL)

## Relations
- Asynchronous Event Loop → USES → mpsc unbounded channel
- Event Streamer Task → USES → crossterm::event::EventStream
- Event Streamer Task → USES → Event enum
- Event Streamer Task → PART_OF → mpsc unbounded channel
- Background Worker Tasks → USES → Action::FileLoaded(Data)
- Main UI Loop → USES → EventHandler
- Main UI Loop → USES → Action::FileLoaded(Data)
- Main UI Loop → USES → Component trait
- EventHandler → RELATED_TO → crossterm::event::EventStream
- EventHandler → USES → Action::FileLoaded(Data)
- App::run() → USES → mpsc unbounded channel
- App::run() → USES → EventHandler
- App::run() → USES → Component trait
- src/main.rs → USES → color-eyre
- src/main.rs → USES → tracing logger
- src/main.rs → USES → App::run()
- src/app.rs → USES → mpsc unbounded channel
- src/app.rs → USES → Component trait
- src/tui.rs → USES → crossterm
- src/tui.rs → USES → EventHandler
- src/action.rs → DEFINES → Action::FileLoaded(Data)
- src/components/mod.rs → USES → Component trait
- src/components/mod.rs → EXPOSES → Component trait
- src/components/log_viewer.rs → IMPLEMENTS → Component trait
- ratatui-themekit → USES → Catppuccin
- ratatui-themekit → USES → Nord
- ratatui-themekit → USES → Tokyo Night
- ratatui-themekit → USES → Dracula
- Flexbox → RELATED_TO → Constraint variants
- crossterm backend → USES → MouseEventKind::Down(MouseButton::Left)
- MouseEventKind::Down(MouseButton::Left) → USES → Position
- MouseEventKind::Down(MouseButton::Left) → USES → Rect
- Rust tracing ecosystem → USES → tracing_appender::non_blocking
- tui-logger → USES → Rust tracing ecosystem
- std::panic::set_hook → RELATED_TO → Raw Mode
- std::panic::set_hook → RELATED_TO → Alternate Screen
- color_eyre::config::HookBuilder → PART_OF → color-eyre
