---
type: entity
title: OpenAI API
created: 2026-06-09
updated: 2026-06-10
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# OpenAI API

Type: SYSTEM

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro06|resilient-rust-based-mcp-client-and-llm-orchestrat-micro06]] (2026-06-09)
- Has a strict reference specification for arguments field.
- Specification requires arguments field to be a string containing serialized JSON.
- Cloud architecture supports returning a JSON array containing multiple function objects.

## From [[resilient-rust-based-mcp-client-and-llm-orchestrat-micro05|resilient-rust-based-mcp-client-and-llm-orchestrat-micro05]] (2026-06-10)
- An industry-standard API specification for tool calling.
- llama-server adheres to this specification to provide compatibility.
