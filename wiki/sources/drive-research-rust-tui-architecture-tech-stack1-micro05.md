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
- [color_eyre::config::HookBuilder](/entities/color-eyre-config-hookbuilder.md) (TOOL)
- [tracing_appender::non_blocking](/entities/tracing-appender-non-blocking.md) (TOOL)
- [Flexbox](/entities/flexbox.md) (CONCEPT)
- [Rust tracing ecosystem](/entities/rust-tracing-ecosystem.md) (TOOL)
- [Alternate Screen](/entities/alternate-screen.md) (CONCEPT)
- [tui-logger](/entities/tui-logger.md) (TOOL)
- [src/main.rs](/entities/src-main-rs.md) (SYSTEM)
- [Asynchronous Event Loop](/entities/asynchronous-event-loop.md) (CONCEPT)
- [MouseEventKind::Down(MouseButton::Left)](/entities/mouseeventkind-down-mousebutton-left.md) (CONCEPT)
- [ratatui-themekit](/entities/ratatui-themekit.md) (TOOL)
- [Background Worker Tasks](/entities/background-worker-tasks.md) (SYSTEM)
- [EventHandler](/entities/eventhandler.md) (SYSTEM)
- [src/config.rs](/entities/src-config-rs.md) (SYSTEM)
- [std::panic::set_hook](/entities/std-panic-set-hook.md) (TOOL)
- [mpsc unbounded channel](/entities/mpsc-unbounded-channel.md) (SYSTEM)
- [tachyonfx](/entities/tachyonfx.md) (TOOL)
- [crossterm::event::EventStream](/entities/crossterm-event-eventstream.md) (TOOL)
- [println!](/entities/println.md) (TOOL)
- [Clear widget](/entities/clear-widget.md) (TOOL)
- [src/components/mod.rs](/entities/src-components-mod-rs.md) (SYSTEM)
- [log::info!](/entities/log-info.md) (TOOL)
- [crossterm backend](/entities/crossterm-backend.md) (TOOL)
- [App::run()](/entities/app-run.md) (SYSTEM)
- [Focusable trait](/entities/focusable-trait.md) (CONCEPT)
- [Event enum](/entities/event-enum.md) (CONCEPT)
- [Constraint variants](/entities/constraint-variants.md) (CONCEPT)
- [src/components/log_viewer.rs](/entities/src-components-log-viewer-rs.md) (SYSTEM)
- [Component trait](/entities/component-trait.md) (CONCEPT)
- [Nord](/entities/nord.md) (CONCEPT)
- [src/tui.rs](/entities/src-tui-rs.md) (SYSTEM)
- [Position](/entities/position.md) (CONCEPT)
- [color-eyre](/entities/color-eyre.md) (TOOL)
- [Dracula](/entities/dracula.md) (CONCEPT)
- [Style::default().fg(Color::Rgb(x,y,z))](/entities/style-default-fg-color-rgb-x-y-z.md) (TOOL)
- [Event Streamer Task](/entities/event-streamer-task.md) (SYSTEM)
- [ratatui::restore()](/entities/ratatui-restore.md) (TOOL)
- [Main UI Loop](/entities/main-ui-loop.md) (SYSTEM)
- [Truecolor](/entities/truecolor.md) (CONCEPT)
- [drive-research-rust-tui-architecture-tech-stack1](/entities/drive-research-rust-tui-architecture-tech-stack1.md) (PROJECT)
- [LogViewer](/entities/logviewer.md) (SYSTEM)
- [Catppuccin](/entities/catppuccin.md) (CONCEPT)
- [Tokyo Night](/entities/tokyo-night.md) (CONCEPT)
- [Action::FileLoaded(Data)](/entities/action-fileloaded-data.md) (CONCEPT)
- [ratatui-interact](/entities/ratatui-interact.md) (TOOL)
- [dbg!](/entities/dbg.md) (TOOL)
- [Raw Mode](/entities/raw-mode.md) (CONCEPT)
- [Rect](/entities/rect.md) (CONCEPT)
- [src/app.rs](/entities/src-app-rs.md) (SYSTEM)
- [src/action.rs](/entities/src-action-rs.md) (SYSTEM)
- [tracing logger](/entities/tracing-logger.md) (TOOL)

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
