---
type: entity
title: UTF-16
created: 2026-06-09
updated: 2026-06-09
sources: 3
tags: []
status: draft
gzmo_synthetic: true
---



# UTF-16

Type: CONCEPT

## From [drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03](/entities/drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03.md) (2026-06-09)
- JavaScript strings are encoded in UTF-16, whereas C systems overwhelmingly expect null-terminated UTF-8 strings.
- When instantiated with a pointer, CString automatically scans native memory for the \0 terminator, transcodes the bytes to UTF-16, and crucially, clones the data.

## From [drive-research-bun-file-parsing-dependency-shortlist-micro02](/entities/drive-research-bun-file-parsing-dependency-shortlist-micro02.md) (2026-06-09)
- Character encoding frequently produced by legacy enterprise systems.

## From [drive-research-bun-typescript-performance-tips-micro03](/entities/drive-research-bun-typescript-performance-tips-micro03.md) (2026-06-09)
- JavaScript strings are encoded in UTF-16.
