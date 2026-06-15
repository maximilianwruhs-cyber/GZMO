---
type: entity
title: html-to-text
created: 2026-06-09
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# html-to-text

Type: TOOL

## From [[drive-research-bun-file-parsing-dependency-shortlist-micro01|drive-research-bun-file-parsing-dependency-shortlist-micro01]] (2026-06-09)
- primary package for HTML to text conversion
- advanced selectors can be slow
- complex CSS-driven layouts or tabular data may transpose rigidly or incorrectly when converted to text

## From [[drive-research-bun-file-parsing-dependency-shortlist-micro02|drive-research-bun-file-parsing-dependency-shortlist-micro02]] (2026-06-09)
- Parses HTML structure logically and applies strict formatting rules to yield highly readable plain text.
- Operates under the MIT license.
- Dual-mode package supporting both CommonJS and ES Modules.
- Utilizes a compiled format to maximize performance during batch processing.
- Gracefully handles the distinction between block-level and inline HTML tags.
- Can optionally render anchor tags alongside their href attributes.
- Moderate bundle size (approximately 2 MB).
- Struggles with legacy HTML emails or archaic web pages that use deeply nested <table> structures for visual layout purposes.
