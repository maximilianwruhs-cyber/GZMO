---
type: entity
title: Raw Mode
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Raw Mode

Type: CONCEPT

## From [[drive-research-rust-tui-architecture-tech-stack1-micro05|drive-research-rust-tui-architecture-tech-stack1-micro05]] (2026-06-09)
- Terminal emulator is placed in this mode for TUI operation.
- Suppresses input echoing, line buffering, and standard signal processing.
- Requires explicit disabling before application exit.
