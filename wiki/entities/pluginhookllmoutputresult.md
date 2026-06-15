---
type: entity
title: PluginHookLlmOutputResult
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# PluginHookLlmOutputResult

Type: CONCEPT

## From [[architectural-analysis-of-the-openclaw-ai-plugin-s|architectural-analysis-of-the-openclaw-ai-plugin-s]] (2026-06-08)
- Plugins can return this to modify the output payload before it is processed by the orchestration layer.
- Properties include assistantText/assistantTexts and usage.
- It is essential for post-processing constraints, such as output scanning protocols or rehydrating masked PII tokens.
