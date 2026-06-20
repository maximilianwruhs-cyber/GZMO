---
type: entity
title: PluginHookLlmInputResult
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# PluginHookLlmInputResult

Type: CONCEPT

## From [architectural-analysis-of-the-openclaw-ai-plugin-s](/entities/architectural-analysis-of-the-openclaw-ai-plugin-s.md) (2026-06-08)
- The callback for the llm_input hook can return this type to override critical generation parameters.
- It allows returning an object containing model, provider, systemPrompt, historyMessages, headers, intercept, and metadata.
- It enables dynamic cost-routing, system prompt modification, context compression, and PII stripping.
