---
type: entity
title: epub2
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# epub2

Type: TOOL

## From [[drive-research-bun-file-parsing-dependency-shortlist-micro02|drive-research-bun-file-parsing-dependency-shortlist-micro02]] (2026-06-09)
- Provides a robust, event-driven interface for deconstructing EPUB containers in modern JavaScript environments.
- Operates under the ISC license.
- Processes the ZIP structures purely in memory without external network calls.
- Returns the raw XHTML of the chapters, requiring downstream processing.
- Reliance on callback-driven events requires manual Promise wrapping.
- Inherently cannot bypass or decrypt files protected by Digital Rights Management (DRM).
- Standard for EPUB format.
