---
type: entity
title: CString
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# CString

Type: TOOL

## From [drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03](/entities/drive-research-advanced-typescript-execution-in-the-bun-runtime-micro03.md) (2026-06-09)
- Bun provides the CString class, which extends the native String object.
- When instantiated with a pointer, CString automatically scans native memory for the \0 terminator, transcodes the bytes to UTF-16, and crucially, clones the data.
- Because the data is cloned, developers can safely invoke native free() functions on the pointer immediately after instantiation without corrupting the JavaScript string.

## From [drive-research-bun-typescript-performance-tips-micro03](/entities/drive-research-bun-typescript-performance-tips-micro03.md) (2026-06-09)
- Bun provides this class, which extends the native String object.
- When instantiated with a pointer, CString scans native memory for the \0 terminator.
- Transcodes bytes to UTF-16 and clones the data.
