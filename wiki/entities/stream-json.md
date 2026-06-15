---
type: entity
title: stream-json
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# stream-json

Type: TOOL

## From [[drive-research-bun-file-parsing-dependency-shortlist-micro02|drive-research-bun-file-parsing-dependency-shortlist-micro02]] (2026-06-09)
- Purely streaming parser that might be required for JSON files exceeding available RAM.
- Ubiquitous, highly standardized unformatted primitive.
- Parsing JSON data into memory operates at the core engine level via JSON.parse().
- Parsing massive, multi-gigabyte JSON payloads is a synchronous operation that will aggressively block the JavaScriptCore main thread.
- In extreme edge cases where JSON files exceed available RAM, a purely streaming parser might be required.
