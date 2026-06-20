---
type: entity
title: Badge API
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Badge API

Type: SYSTEM

## From [drive-research-creating-a-comprehensive-readmemd-micro02](/entities/drive-research-creating-a-comprehensive-readmemd-micro02.md) (2026-06-09)
- Badges are constructed using a highly standardized API endpoint format.
- URL structure for static badges: https://img.shields.io/badge/<LABEL>-<MESSAGE>-<COLOR>.
- Escape sequences required for dashes and underscores: %20 or _ for space, __ for literal underscore, -- for literal dash.
- Message-only template available by dropping the label segment.
