---
type: entity
title: Meta-Zod-Schema
created: 2026-06-09
updated: 2026-06-09
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Meta-Zod-Schema

Type: TOOL

## From [[prompt-agent-engineering-part5-micro05|prompt-agent-engineering-part5-micro05]] (2026-06-09)
- Used for validation of generated code
- Used for input/output schema definition
- Used for meta-skill schema
- Used for parsing generated JSON against meta-skill schema
- Used for validating generated code's output against outputSchema in dry-run
- Strict schema for LLM generation
- Forces model to return an exact JSON object
- Defines name, description, input/output schema, and code string
