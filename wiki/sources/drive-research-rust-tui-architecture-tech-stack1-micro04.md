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
- [[iocraft|Iocraft]] (TOOL)
- [[ratatui-image|ratatui-image]] (TOOL)
- [[the-elm-architecture|The Elm Architecture]] (CONCEPT)
- [[ansi-escape-sequences|ANSI escape sequences]] (CONCEPT)
- [[tui-rs|tui-rs]] (TOOL)
- [[immediate-mode-rendering-paradigm|immediate-mode rendering paradigm]] (CONCEPT)
- [[double-buffering|double-buffering]] (CONCEPT)
- [[unicode-halfblocks|Unicode Halfblocks]] (CONCEPT)
- [[action|Action]] (CONCEPT)
- [[termion|termion]] (TOOL)
- [[tui-scrollview|tui-scrollview]] (TOOL)
- [[rust|Rust]] (CONCEPT)
- [[ratatui-interact|ratatui-interact]] (TOOL)
- [[tree-sitter|tree-sitter]] (TOOL)
- [[tui-tree-widget|tui-tree-widget]] (TOOL)
- [[bubbletea|BubbleTea]] (TOOL)
- [[cursive|Cursive]] (TOOL)
- [[termwiz|termwiz]] (TOOL)
- [[tokio|tokio]] (SYSTEM)
- [[component-architecture|Component Architecture]] (CONCEPT)
- [[multiline-input|multiline_input]] (TOOL)
- [[command-line-interfaces|Command Line Interfaces]] (CONCEPT)
- [[sixel-protocol|Sixel Protocol]] (CONCEPT)
- [[ratatui-code-editor|ratatui-code-editor]] (TOOL)
- [[crossterm|crossterm]] (TOOL)
- [[tui-textarea|tui-textarea]] (TOOL)
- [[iterm2-protocol|iTerm2 Protocol]] (CONCEPT)
- [[terminal-user-interfaces|Terminal User Interfaces]] (CONCEPT)
- [[kitty-graphics-protocol|Kitty Graphics Protocol]] (CONCEPT)
- [[statefulimage|StatefulImage]] (TOOL)

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
