---
type: entity
title: Alternate Screen
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Alternate Screen

Type: CONCEPT

## From [[drive-research-rust-tui-architecture-tech-stack1-micro05|drive-research-rust-tui-architecture-tech-stack1-micro05]] (2026-06-09)
- TUI usually operates on this buffer.
- Avoids overwriting the user's command history.
- Requires explicit leaving before application exit.
