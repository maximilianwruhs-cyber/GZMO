---
type: entity
title: Code Block Directives
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Code Block Directives

Type: CONCEPT

## From [[high-fidelity-markdown-engineering-and-ast-process|high-fidelity-markdown-engineering-and-ast-process]] (2026-06-08)
- Represents a block of preformatted text (fenced code block).
- Is a Literal node.
- Actual code snippet stored in its value string.
- Includes optional lang and meta strings.
- Stringifier evaluates code block contents and may output empty or un-flagged code blocks using older indented code format.
- Fences boolean must be set to true to guarantee GZMO's injected code remains within clear fences.
- Fence option refines this by dictating whether to use backticks or tildes as the primary boundary.
