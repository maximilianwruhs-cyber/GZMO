---
type: entity
title: Context Files
created: 2026-06-08
updated: 2026-06-08
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# Context Files

Type: CONCEPT

## From [[drive-research-hermes-agent-prompt-builder-analysis|drive-research-hermes-agent-prompt-builder-analysis]] (2026-06-08)
- Includes AGENTS.md, .cursorrules, etc.
- Injects localized, repository-specific rules.
- If SOUL.md was not loaded in Layer 1, it may be loaded here.

## From [[drive-research-inside-the-pi-coding-agent-optimization-isn|drive-research-inside-the---pi---coding-agent--optimization--isn]] (2026-06-08)
- Used to feed necessary source code or API definitions into a session.
- Helps prevent context drift.

## From [[drive-research-optimizing-pi-coding-agent-performance|drive-research-optimizing-pi-coding-agent-performance]] (2026-06-08)
- Used for targeted context injection.
- Prefix specific paths with the @ symbol (e.g., pi @src/core/lib.rs).
- Only the exact target content is parsed and indexed into the startup message.
