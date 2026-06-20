---
type: entity
title: Top-K
created: 2026-06-08
updated: 2026-06-09
sources: 2
tags: []
status: draft
gzmo_synthetic: true
---


# Top-K

Type: CONCEPT

## From [drive-research-crafting-a-creative-agent-prompt](/entities/drive-research-crafting-a-creative-agent-prompt.md) (2026-06-08)
- Restricts the model to a fixed, absolute number of the most likely next tokens.
- Setting a value between 40 and 50 acts as an additional safety net.

## From [drive-research-llamacpp-optimization-blueprint-micro03](/entities/drive-research-llamacpp-optimization-blueprint-micro03.md) (2026-06-09)
- A legacy sampler that arbitrarily truncates the vocabulary to a fixed number of tokens.
- Must be disabled when using Min-P.
