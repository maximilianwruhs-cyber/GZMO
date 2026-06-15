---
type: entity
title: _anthropic_prompt_cache_policy()
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# _anthropic_prompt_cache_policy()

Type: TOOL

## From [[drive-research-hermes-anthropic-openrouter-cache-investigation|drive-research-hermes-anthropic-openrouter-cache-investigation]] (2026-06-08)
- This function in Hermes code is designed to detect when a Claude model is called via OpenRouter.
- It correctly returns the tuple (True, False) signaling that caching should be applied using an 'envelope' layout.
- It is an API provider whose caching mechanisms are investigated.
- It defines the cache_control parameter for prompt caching.
- Its API strictly requires cache_control markers on top-level system content blocks in the native anthropic_messages wire format.
- Provides prompt caching parameters like cache_control.
- Uses a rolling three-message window strategy.
