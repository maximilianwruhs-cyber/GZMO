---
type: source
title: tui-framework
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: stable
gzmo_synthetic: true
---

# tui-framework

Ingested source summary (2026-06-08).

## Entities
- [[broot|broot]] (TOOL)
- [[c-l-o-s-e-r-parameters|C.L.O.S.E.R. parameters]] (CONCEPT)
- [[vte|vte]] (TOOL)
- [[vscodium-extension-ecosystem|VSCodium extension ecosystem]] (CONCEPT)
- [[portable-pty|portable-pty]] (TOOL)
- [[vscode-window-createoutputchannel|vscode.window.createOutputChannel]] (TOOL)
- [[fsevents|FSEvents]] (SYSTEM)
- [[tui-framework|TUI Framework]] (CONCEPT)
- [[vscode-treedataprovider|vscode.TreeDataProvider]] (TOOL)
- [[eof|EOF]] (CONCEPT)
- [[vscode-workspace-createfilesystemwatcher|vscode.workspace.createFileSystemWatcher]] (TOOL)
- [[node-js-extension-host|Node.js Extension Host]] (SYSTEM)
- [[ide-kdl|ide.kdl]] (CONCEPT)
- [[google-takeout|Google Takeout]] (TOOL)
- [[tui-terminal-user-interface-workspace-anwendung|TUI (Terminal User Interface) Workspace-Anwendung]] (CONCEPT)
- [[tree-of-thoughts-tot|Tree-of-Thoughts (ToT)]] (CONCEPT)
- [[apt|APT]] (TOOL)
- [[textual|Textual]] (TOOL)
- [[creack-pty|creack/pty]] (TOOL)
- [[esbuild|esbuild]] (TOOL)
- [[vscode-window-createterminal|vscode.window.createTerminal]] (TOOL)
- [[linux-unix-system-calls|Linux/UNIX system calls]] (CONCEPT)
- [[ubuntu-24-04-lts|Ubuntu 24.04 LTS]] (SYSTEM)
- [[ratatui|Ratatui]] (TOOL)
- [[node-pty|node-pty]] (TOOL)
- [[xterm-js|xterm.js]] (TOOL)
- [[tiocswinsz|TIOCSWINSZ]] (CONCEPT)
- [[webviewpanel|WebviewPanel]] (TOOL)
- [[csp|CSP]] (CONCEPT)
- [[vscode-window-createtreeview|vscode.window.createTreeView]] (TOOL)
- [[inotify|inotify]] (SYSTEM)
- [[notebooklm|NotebookLM]] (TOOL)
- [[gemini|Gemini]] (PERSON)
- [[zellij|Zellij]] (TOOL)
- [[s2a-filter|S2A Filter]] (TOOL)
- [[bash|bash]] (SYSTEM)
- [[split-js|split.js]] (TOOL)
- [[python|Python]] (CONCEPT)
- [[ansi-escape-codes|ANSI escape codes]] (CONCEPT)
- [[btop|btop]] (TOOL)
- [[bubbletea|Bubbletea]] (TOOL)
- [[rust|Rust]] (CONCEPT)
- [[webview-bridge|Webview Bridge]] (CONCEPT)
- [[webview-architecture|Webview architecture]] (SYSTEM)
- [[yazi|yazi]] (TOOL)
- [[go|Go]] (CONCEPT)
- [[senior-software-architect|Senior Software Architect]] (PERSON)
- [[vscode-webview-ui-toolkit|@vscode/webview-ui-toolkit]] (TOOL)
- [[vite|Vite]] (TOOL)
- [[native-api-first|Native API First]] (CONCEPT)
- [[dev-ptmx|/dev/ptmx]] (SYSTEM)

## Relations
- TUI Framework → PART_OF → Google Takeout
- TUI Framework → PART_OF → NotebookLM
- VSCodium extension ecosystem → PART_OF → Node.js Extension Host
- VSCodium extension ecosystem → PART_OF → Webview architecture
- Node.js Extension Host → USES → node-pty
- Webview architecture → USES → xterm.js
- Webview architecture → USES → @vscode/webview-ui-toolkit
- Webview architecture → USES → split.js
- Gemini → RELATED_TO → C.L.O.S.E.R. parameters
- TUI (Terminal User Interface) Workspace-Anwendung → RELATED_TO → Rust
- TUI (Terminal User Interface) Workspace-Anwendung → RELATED_TO → Go
- TUI (Terminal User Interface) Workspace-Anwendung → RELATED_TO → Zellij
- portable-pty → RELATED_TO → Linux/UNIX system calls
- portable-pty → RELATED_TO → /dev/ptmx
- portable-pty → RELATED_TO → TIOCSWINSZ
- ANSI escape codes → RELATED_TO → portable-pty
- Linux/UNIX system calls → USES → /dev/ptmx
- Linux/UNIX system calls → USES → TIOCSWINSZ
- Linux/UNIX system calls → USES → inotify
- Linux/UNIX system calls → USES → FSEvents
- Ratatui → USES → portable-pty
- Ratatui → USES → vte
- Bubbletea → USES → creack/pty
- Zellij → USES → ide.kdl
- ide.kdl → PART_OF → Zellij
- TUI (Terminal User Interface) Workspace-Anwendung → RELATED_TO → portable-pty
- TUI (Terminal User Interface) Workspace-Anwendung → RELATED_TO → ANSI escape codes
- TUI (Terminal User Interface) Workspace-Anwendung → RELATED_TO → Python
- S2A Filter → RELATED_TO → TUI Framework
- Tree-of-Thoughts (ToT) → RELATED_TO → TUI Framework
- Zellij → USES → yazi
- Zellij → USES → btop
- Zellij → USES → broot
- Ubuntu 24.04 LTS → RELATED_TO → bash
- Ubuntu 24.04 LTS → USES → APT
- APT → USES → zellij
- APT → USES → broot
- APT → USES → btop
