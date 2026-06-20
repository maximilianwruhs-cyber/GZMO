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
- [broot](/entities/broot.md) (TOOL)
- [C.L.O.S.E.R. parameters](/entities/c-l-o-s-e-r-parameters.md) (CONCEPT)
- [vte](/entities/vte.md) (TOOL)
- [VSCodium extension ecosystem](/entities/vscodium-extension-ecosystem.md) (CONCEPT)
- [portable-pty](/entities/portable-pty.md) (TOOL)
- [vscode.window.createOutputChannel](/entities/vscode-window-createoutputchannel.md) (TOOL)
- [FSEvents](/entities/fsevents.md) (SYSTEM)
- [TUI Framework](/entities/tui-framework.md) (CONCEPT)
- [vscode.TreeDataProvider](/entities/vscode-treedataprovider.md) (TOOL)
- [EOF](/entities/eof.md) (CONCEPT)
- [vscode.workspace.createFileSystemWatcher](/entities/vscode-workspace-createfilesystemwatcher.md) (TOOL)
- [Node.js Extension Host](/entities/node-js-extension-host.md) (SYSTEM)
- [ide.kdl](/entities/ide-kdl.md) (CONCEPT)
- [Google Takeout](/entities/google-takeout.md) (TOOL)
- [TUI (Terminal User Interface) Workspace-Anwendung](/entities/tui-terminal-user-interface-workspace-anwendung.md) (CONCEPT)
- [Tree-of-Thoughts (ToT)](/entities/tree-of-thoughts-tot.md) (CONCEPT)
- [APT](/entities/apt.md) (TOOL)
- [Textual](/entities/textual.md) (TOOL)
- [creack/pty](/entities/creack-pty.md) (TOOL)
- [esbuild](/entities/esbuild.md) (TOOL)
- [vscode.window.createTerminal](/entities/vscode-window-createterminal.md) (TOOL)
- [Linux/UNIX system calls](/entities/linux-unix-system-calls.md) (CONCEPT)
- [Ubuntu 24.04 LTS](/entities/ubuntu-24-04-lts.md) (SYSTEM)
- [Ratatui](/entities/ratatui.md) (TOOL)
- [node-pty](/entities/node-pty.md) (TOOL)
- [xterm.js](/entities/xterm-js.md) (TOOL)
- [TIOCSWINSZ](/entities/tiocswinsz.md) (CONCEPT)
- [WebviewPanel](/entities/webviewpanel.md) (TOOL)
- [CSP](/entities/csp.md) (CONCEPT)
- [vscode.window.createTreeView](/entities/vscode-window-createtreeview.md) (TOOL)
- [inotify](/entities/inotify.md) (SYSTEM)
- [NotebookLM](/entities/notebooklm.md) (TOOL)
- [Gemini](/entities/gemini.md) (PERSON)
- [Zellij](/entities/zellij.md) (TOOL)
- [S2A Filter](/entities/s2a-filter.md) (TOOL)
- [bash](/entities/bash.md) (SYSTEM)
- [split.js](/entities/split-js.md) (TOOL)
- [Python](/entities/python.md) (CONCEPT)
- [ANSI escape codes](/entities/ansi-escape-codes.md) (CONCEPT)
- [btop](/entities/btop.md) (TOOL)
- [Bubbletea](/entities/bubbletea.md) (TOOL)
- [Rust](/entities/rust.md) (CONCEPT)
- [Webview Bridge](/entities/webview-bridge.md) (CONCEPT)
- [Webview architecture](/entities/webview-architecture.md) (SYSTEM)
- [yazi](/entities/yazi.md) (TOOL)
- [Go](/entities/go.md) (CONCEPT)
- [Senior Software Architect](/entities/senior-software-architect.md) (PERSON)
- [@vscode/webview-ui-toolkit](/entities/vscode-webview-ui-toolkit.md) (TOOL)
- [Vite](/entities/vite.md) (TOOL)
- [Native API First](/entities/native-api-first.md) (CONCEPT)
- [/dev/ptmx](/entities/dev-ptmx.md) (SYSTEM)

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
