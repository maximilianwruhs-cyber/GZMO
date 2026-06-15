---
type: entity
title: Kilocode pattern
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# Kilocode pattern

Type: CONCEPT

## From [[drive-research-hermes-agent-prompt-builder-analysis|drive-research-hermes-agent-prompt-builder-analysis]] (2026-06-08)
- Leverages strict XML segmentation to command LLM attention.
- Injects critical, state-specific directives into the active user message position.
- Requires breaking down the monolithic prompt_parts string into discrete, taggable objects.
- Used to distribute dynamic instructions directly to the generation point.
- Enables the architecture to maintain high instruction adherence.
