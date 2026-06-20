---
type: entity
title: apply_anthropic_cache_control()
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# apply_anthropic_cache_control()

Type: CONCEPT

## From [drive-research-hermes-anthropic-openrouter-cache-investigation](/entities/drive-research-hermes-anthropic-openrouter-cache-investigation.md) (2026-06-08)
- A prompt caching parameter provided by providers.
- Used by Hermes framework.
- This function in prompt_caching.py can theoretically process both ephemeral and 1-hour TTL values.
- Its theoretical capability is undermined by hardcoding in run_agent.py.
