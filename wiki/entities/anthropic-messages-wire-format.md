---
type: entity
title: anthropic_messages-Wire-Format
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# anthropic_messages-Wire-Format

Type: CONCEPT

## From [[drive-research-hermes-anthropic-openrouter-cache-investigation|drive-research-hermes-anthropic-openrouter-cache-investigation]] (2026-06-08)
- Anthropic's backend strictly requires cache_control markers to be on top-level system content blocks in this format.
- The chat_completions API mode does not adhere to this format.
