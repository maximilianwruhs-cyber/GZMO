---
type: entity
title: Markdown Abstract Syntax Tree (MDAST)
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Markdown Abstract Syntax Tree (MDAST)

Type: CONCEPT

## From [[high-fidelity-markdown-engineering-and-ast-process|high-fidelity-markdown-engineering-and-ast-process]] (2026-06-08)
- Flawlessly manages Markdown syntax.
- Is translated into the HTML Abstract Syntax Tree (hast) specification.
- Is the target specification for raw text body.
- Specification for the raw text body.
- Transitioning to this guarantees programmatic certainty.
- Leveraged by the GZMO architecture.
- An AST manipulation plugin.
- Used to find specific non-standard syntaxes.
- Converts them into custom nodes or raw HTML nodes.
- Rigorous, strongly typed specification for representing Markdown constructs.
- Represents Markdown as a predictable, traversable JSON object.
- Implements the Universal Syntax Tree (unist) specification.
- All nodes adhere to a strict set of programmatic interfaces.
- Low-level utility used by remark-stringify.
- Handles rigorous formatting logic during serialization.
- API specifications were synthesized from.
- Used in conjunction with remark-stringify.
- Utility for extracting concatenated plaintext value of nodes.
- Used to evaluate the full plaintext value of a heading.
- Package that introduces an arbitrary extension syntax to Markdown.
- Functions similarly to shortcodes.
- Directives are parsed into distinct, strongly typed nodes within the AST.
