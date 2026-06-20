---
type: entity
title: experimental_repairToolCall
created: 2026-06-08
updated: 2026-06-08
sources: 1
tags: []
status: draft
gzmo_synthetic: true
---

# experimental_repairToolCall

Type: CONCEPT

## From [migrating-openclaw-to-vercel-ai-sdk-for-local-llm](/entities/migrating-openclaw-to-vercel-ai-sdk-for-local-llm.md) (2026-06-08)
- A function introduced by the SDK.
- Catches specific exceptions like NoSuchToolError or InvalidToolArgumentsError.
- Allows developers to fix broken JSON payloads automatically and inject the repaired tool call back into the execution stream.
