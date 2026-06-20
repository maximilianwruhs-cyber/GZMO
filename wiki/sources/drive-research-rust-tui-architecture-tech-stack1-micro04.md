---
type: source
title: drive-research-rust-tui-architecture-tech-stack1-micro04
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# drive-research-rust-tui-architecture-tech-stack1-micro04

Ingested source summary (2026-06-09).

## Entities
- [Iocraft](/entities/iocraft.md) (TOOL)
- [ratatui-image](/entities/ratatui-image.md) (TOOL)
- [The Elm Architecture](/entities/the-elm-architecture.md) (CONCEPT)
- [ANSI escape sequences](/entities/ansi-escape-sequences.md) (CONCEPT)
- [tui-rs](/entities/tui-rs.md) (TOOL)
- [immediate-mode rendering paradigm](/entities/immediate-mode-rendering-paradigm.md) (CONCEPT)
- [double-buffering](/entities/double-buffering.md) (CONCEPT)
- [Unicode Halfblocks](/entities/unicode-halfblocks.md) (CONCEPT)
- [Action](/entities/action.md) (CONCEPT)
- [termion](/entities/termion.md) (TOOL)
- [tui-scrollview](/entities/tui-scrollview.md) (TOOL)
- [Rust](/entities/rust.md) (CONCEPT)
- [ratatui-interact](/entities/ratatui-interact.md) (TOOL)
- [tree-sitter](/entities/tree-sitter.md) (TOOL)
- [tui-tree-widget](/entities/tui-tree-widget.md) (TOOL)
- [BubbleTea](/entities/bubbletea.md) (TOOL)
- [Cursive](/entities/cursive.md) (TOOL)
- [termwiz](/entities/termwiz.md) (TOOL)
- [tokio](/entities/tokio.md) (SYSTEM)
- [Component Architecture](/entities/component-architecture.md) (CONCEPT)
- [multiline_input](/entities/multiline-input.md) (TOOL)
- [Command Line Interfaces](/entities/command-line-interfaces.md) (CONCEPT)
- [Sixel Protocol](/entities/sixel-protocol.md) (CONCEPT)
- [ratatui-code-editor](/entities/ratatui-code-editor.md) (TOOL)
- [crossterm](/entities/crossterm.md) (TOOL)
- [tui-textarea](/entities/tui-textarea.md) (TOOL)
- [iTerm2 Protocol](/entities/iterm2-protocol.md) (CONCEPT)
- [Terminal User Interfaces](/entities/terminal-user-interfaces.md) (CONCEPT)
- [Kitty Graphics Protocol](/entities/kitty-graphics-protocol.md) (CONCEPT)
- [StatefulImage](/entities/statefulimage.md) (TOOL)

## Relations
- Rust → RELATED_TO → Terminal User Interfaces
- Terminal User Interfaces → RELATED_TO → Command Line Interfaces
- ratatui-image → RELATED_TO → tui-rs
- ratatui-image → USES → immediate-mode rendering paradigm
- ratatui-image → USES → double-buffering
- ratatui-image → USES → ANSI escape sequences
- ratatui-image → USES → Crossterm
- ratatui-image → USES → Termwiz
- ratatui-image → USES → Termion
- BubbleTea → USES → The Elm Architecture
- Crossterm → USES → tokio
- ratatui-code-editor → USES → tree-sitter
- ratatui-image → USES → Sixel Protocol
- ratatui-image → USES → Kitty Graphics Protocol
- ratatui-image → USES → iTerm2 Protocol
- ratatui-image → USES → Unicode Halfblocks
