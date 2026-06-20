---
type: entity
title: Node interface
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Node interface

Type: CONCEPT

## From [high-fidelity-markdown-engineering-and-ast-process](/entities/high-fidelity-markdown-engineering-and-ast-process.md) (2026-06-08)
- Baseline interface implemented by every syntactic unit in MDAST.
- Guarantees every object contains a type field and an optional position field.
- Position field tracks the precise location of markup within the source file.
