---
type: entity
title: Token-Optimized Data Serialization Formats
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Token-Optimized Data Serialization Formats

Type: CONCEPT

## From [drive-research-token-efficient-bol-processing-architecture](/entities/drive-research-token-efficient-bol-processing-architecture.md) (2026-06-08)
- Dictates token consumption based on the structural format of data supplied in the LLM prompt.
- Converting from verbose JSON to concise YAML or custom delimiter-separated schemas drastically minimizes the token footprint.
- YAML relies heavily on semantic whitespace, line breaks, and indentation, which tokenizers process significantly more efficiently than nested JSON syntax.
