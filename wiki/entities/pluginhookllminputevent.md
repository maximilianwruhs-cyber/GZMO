---
type: entity
title: PluginHookLlmInputEvent
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# PluginHookLlmInputEvent

Type: CONCEPT

## From [architectural-analysis-of-the-openclaw-ai-plugin-s](/entities/architectural-analysis-of-the-openclaw-ai-plugin-s.md) (2026-06-08)
- The llm_input hook fires, passing this object to the registered handler.
- It exposes critical context to the plugin, including runId, sessionId, provider, model, prompt, and historyMessages.
- It notably omits inference-specific tuning parameters like temperature or max_tokens.
