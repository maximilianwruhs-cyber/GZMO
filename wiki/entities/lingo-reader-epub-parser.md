---
type: entity
title: '@lingo-reader/epub-parser'
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# @lingo-reader/epub-parser

Type: CONCEPT

## From [drive-research-bun-file-parsing-dependency-shortlist-micro01](/entities/drive-research-bun-file-parsing-dependency-shortlist-micro01.md) (2026-06-09)
- async event-driven architecture requires manual Promise wrapping
- cannot bypass Digital Rights Management (DRM) encryption
- secondary package for EPUB parsing

## From [drive-research-bun-file-parsing-dependency-shortlist-micro02](/entities/drive-research-bun-file-parsing-dependency-shortlist-micro02.md) (2026-06-09)
- Format (specifically EPUB2 and EPUB3 standards) is a self-contained, offline website wrapped inside a highly structured ZIP archive.
- Contains an Open Package Format (OPF) file that serves as a core manifest.
- Contains an NCX file for the hierarchical table of contents.
- Contains numerous XHTML files representing the actual chapters, sections, and metadata.
- Extracting unified plain text is a two-step orchestration process: archive unpacking and manifest parsing, followed by HTML-to-text extraction.
- A modern alternative to the callback-heavy epub2.
- Provides a more modernized API surface.
- Provides native TypeScript interfaces for the Table of Contents and navigation points.
- MIT licensed.
- Explicitly avoids network dependencies.
- Actively maintained.
