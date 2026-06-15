---
type: entity
title: PluginHookLlmOutputEvent
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# PluginHookLlmOutputEvent

Type: CONCEPT

## From [[architectural-analysis-of-the-openclaw-ai-plugin-s|architectural-analysis-of-the-openclaw-ai-plugin-s]] (2026-06-08)
- Upon receiving the response from the LLM, the llm_output hook fires, providing this object.
- It includes provider, model, assistantTexts, lastAssistant state data, provider usage statistics, durationMs, isRetry flags, retryCount, and metadata.
- Plugins can use this to modify the output payload.
