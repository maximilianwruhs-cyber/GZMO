---
type: entity
title: Focusable interface
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Focusable interface

Type: CONCEPT

## From [[drive-research-building-pi-coding-agent-extensions|drive-research-building-pi-coding-agent-extensions]] (2026-06-08)
- Signals the TUI engine to route standard input (stdin) keystrokes directly to the handleInput method.
- Requires embedding a zero-width APC escape sequence for IME support.

## From [[high-performance-typescript-execution-and-architec-part1-micro06|high-performance-typescript-execution-and-architec-part1-micro06]] (2026-06-09)
- Implemented by components requiring precise keyboard navigation.
- Signals the TUI engine to route stdin keystrokes to the handleInput method.
- Requires embedding a zero-width APC escape sequence (CURSOR_MARKER) for IME support.
